use crate::Anilist;
use crate::Result;

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

/// Anilist query
pub const QUERY: &str = r#"
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

/// Serialized query of Anilist.
#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
    pub data: Data,
}

/// Serialized data of Anilist.
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

/// Serialized media of Anilist.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    id: usize,
    pub title: Title,
    pub description: String,
    pub genres: Vec<String>,
    pub banner_image: String,
    pub studios: Studio,
}

/// Serialized titles of Anilist.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Title {
    pub romaji: String,
    pub english: String,
    pub native: String,
}

/// Serialized studios of Anilist.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Studio {
    pub edges: Vec<Edges>,
}

/// Serialized studio edge of Anilist.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Edges {
    pub is_main: bool,
    pub node: Node,
}

/// Serialized edge node of Anilist studios.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub name: String,
}

impl Data {
    pub async fn download_image(mut self, api: &Anilist, path: &Path) -> Result<Self> {
        use tokio::io::AsyncWriteExt;
        use tokio_stream::StreamExt;

        let resp = api.get_body(&self.media.banner_image).await?;
        let mut body = resp.into_body();

        let name_file = path.join(".metadata").join("thumbnail.jpg");
        let mut file = tokio::fs::File::create(&name_file).await?;

        while let Some(chunk) = body.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        info!("Image downloaded for: {}", self.media.title.english);
        self.set_thumbnail_path(name_file);
        Ok(self)
    }

    pub fn set_thumbnail_path(&mut self, path: PathBuf) -> &mut Self {
        self.thumbnail_path = path;
        self
    }

    pub fn set_id(&mut self, id: usize) -> &mut Self {
        self.id = id;
        self
    }

    pub fn clean_description(&mut self) -> &mut Self {
        let ac =
            AhoCorasick::new(["<b>", "</b>", "<i>", "</i>", "<br>\n<br>", "<br><br>"]).unwrap();
        self.media.description =
            ac.replace_all(&self.media.description, &["", "", "", "", "\n", "\n"]);

        self
    }

    pub fn find_studio(&mut self) -> &mut Self {
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
