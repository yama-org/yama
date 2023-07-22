use crate::backend::title::Title as BTitle;
use crate::Result;
use aho_corasick::AhoCorasick;
use anyhow::bail;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use tracing::info;

const QUERY: &str = r#"
query ($search: String) {
  Media (search: $search, type: ANIME) {
    id,
    title {
      romaji,
      english,
      native,
    },
    description,
    genres,
    bannerImage,
    studios {
        edges {
          isMain,
          node {
            name
          }
        }
      }
  }
}
"#;

pub struct Anilist {
    client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
struct Query {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
    pub media: Media,
    #[serde(skip)]
    pub thumbnail_path: PathBuf,
    #[serde(skip)]
    pub id: usize,
    #[serde(skip)]
    pub studio: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    id: usize,
    pub title: Title,
    pub description: String,
    pub genres: Vec<String>,
    banner_image: String,
    pub studios: Studio,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Title {
    pub romaji: String,
    pub english: String,
    pub native: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Studio {
    pub edges: Vec<Edges>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Edges {
    pub is_main: bool,
    pub node: Node,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub name: String,
}

impl Default for Anilist {
    fn default() -> Anilist {
        Anilist::new()
    }
}

impl Anilist {
    pub fn new() -> Anilist {
        Anilist {
            client: Client::new(),
        }
    }

    async fn query(&self, path: &Path, search: &str, id: usize) -> Result<Data> {
        let json = json!({"query": QUERY, "variables": {"search": search}});
        let resp = self
            .client
            .post("https://graphql.anilist.co/") //BUT WHY IS IT POST???, I must move to hyper as soon as i can!
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await?
            .text()
            .await?;

        let result: Query = serde_json::from_str(&resp)?;
        std::fs::write(path.join(".metadata").join("data.json"), resp)?;

        let data = result.data.download_image(self, path);
        let mut data = data.await?;
        data.set_id(id);
        data.clean_description();
        data.find_studio();

        Ok(data)
    }

    fn cached_query(&self, path: &Path, id: usize) -> Result<Data> {
        let content = std::fs::read_to_string(path.join(".metadata").join("data.json"))?;

        let result: Query = serde_json::from_str(&content)?;

        let mut data = result.data;
        data.set_id(id);
        data.set_thumbnail_path(path.join(".metadata").join("thumbnail.jpg"));
        data.clean_description();
        data.find_studio();

        Ok(data)
    }

    pub async fn try_query(&self, title: &mut BTitle, id: usize) -> Result<()> {
        let path = title.path.as_path();
        let search = &title.name;

        if let Ok(files) = std::fs::read_dir(path.join(".metadata")) {
            let files: Vec<OsString> = files
                .into_iter()
                .flatten()
                .map(|file| file.file_name())
                .collect();

            let was_downloaded = files.contains(&OsString::from("thumbnail.jpg"))
                && files.contains(&OsString::from("data.json"));

            title.data = if was_downloaded {
                self.cached_query(path, id)
            } else {
                self.query(path, search, id).await
            }
            .ok();
        }

        match title.data {
            Some(_) => Ok(()),
            None => bail!("Failed query of: {}", search),
        }
    }

    async fn download_image(&self, url: &str) -> Result<reqwest::Response> {
        let res = self.client.get(url).send().await?;
        Ok(res)
    }
}

impl Data {
    async fn download_image(mut self, api: &Anilist, path: &Path) -> Result<Data> {
        let bytes = api
            .download_image(&self.media.banner_image)
            .await?
            .bytes()
            .await?;

        info!("Image downloaded for: {}", self.media.title.english);

        let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg)?;
        /*if image.width() > 1900 || image.height() > 400 {
            image = image.resize_to_fill(1900, 400, image::imageops::FilterType::Nearest);
        }*/

        let name_file = path.join(".metadata").join("thumbnail.jpg");
        let mut file = std::fs::File::create(&name_file)?;
        image.write_to(&mut file, image::ImageFormat::Jpeg)?;

        self.set_thumbnail_path(name_file);
        Ok(self)
    }

    fn set_thumbnail_path(&mut self, path: PathBuf) -> &mut Self {
        self.thumbnail_path = path;
        self
    }

    fn set_id(&mut self, id: usize) -> &mut Self {
        self.id = id;
        self
    }

    fn clean_description(&mut self) -> &mut Self {
        let ac =
            AhoCorasick::new(["<b>", "</b>", "<i>", "</i>", "<br>\n<br>", "<br><br>"]).unwrap();
        self.media.description =
            ac.replace_all(&self.media.description, &["", "", "", "", "\n", "\n"]);

        self
    }

    fn find_studio(&mut self) -> &mut Self {
        for studio in &self.media.studios.edges {
            if studio.is_main {
                self.studio = studio.node.name.clone();
                break;
            }
        }

        self
    }
}

impl Media {
    pub fn to_str(&self) -> Box<str> {
        format!(
            "Description: {}\n\nGenres: {}",
            self.description.trim(),
            self.genres.join(", ")
        )
        .into_boxed_str()
    }
}
