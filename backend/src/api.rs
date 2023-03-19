use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    ffi::OsString,
    io::Write,
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

type Result<T> = std::result::Result<T, Error>;

pub struct Api {
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

#[derive(Debug)]
pub enum ErrorKind {
    Request(reqwest::Error),
    Parse(serde_json::Error),
    File(std::io::Error),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    title: String,
}

impl Error {
    pub fn new(kind: ErrorKind, title: &str) -> Error {
        Error {
            kind,
            title: title.to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.kind {
            ErrorKind::Request(_) => write!(f, "Could not get requested data for: {}", self.title),
            ErrorKind::Parse(_) => write!(f, "Could not parse data for: {}", self.title),
            ErrorKind::File(_) => write!(f, "Could not create file for: {}", self.title),
        }
    }
}

impl From<reqwest::Error> for ErrorKind {
    fn from(err: reqwest::Error) -> ErrorKind {
        ErrorKind::Request(err)
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> ErrorKind {
        ErrorKind::Parse(err)
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(err: std::io::Error) -> ErrorKind {
        ErrorKind::File(err)
    }
}

impl Default for Api {
    fn default() -> Api {
        Api::new()
    }
}

impl Api {
    pub fn new() -> Api {
        Api {
            client: Client::new(),
        }
    }

    async fn query(&self, path: &Path, search: &str, id: usize) -> Result<Data> {
        let json = json!({"query": QUERY, "variables": {"search": search}});
        let resp = self
            .client
            .post("https://graphql.anilist.co/")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await
            .map_err(|e| Error::new(e.into(), search))?
            .text()
            .await
            .map_err(|e| Error::new(e.into(), search))?;

        info!("Data downloaded for: {}", search);

        let result: Query =
            serde_json::from_str(&resp).map_err(|e| Error::new(e.into(), search))?;

        std::fs::write(path.join(".metadata").join("data.json"), resp)
            .map_err(|e| Error::new(e.into(), search))?;

        let data = result.data.download_image(self, path);
        let mut data = data.await?;
        data.set_id(id);
        data.clean_description();
        data.find_studio();

        Ok(data)
    }

    fn cached_query(&self, path: &Path, search: &str, id: usize) -> Result<Data> {
        let content = std::fs::read_to_string(path.join(".metadata").join("data.json"))
            .map_err(|e| Error::new(e.into(), search))?;

        let result: Query =
            serde_json::from_str(&content).map_err(|e| Error::new(e.into(), search))?;

        let mut data = result.data;
        data.set_id(id);
        data.set_thumbnail_path(path.join(".metadata").join("thumbnail.jpg"));
        data.clean_description();
        data.find_studio();

        Ok(data)
    }

    pub async fn try_query(&self, path: &Path, search: &str, id: usize) -> Result<Data> {
        match std::fs::read_dir(path.join(".metadata")) {
            Ok(files) => {
                let files: Vec<OsString> = files
                    .into_iter()
                    .flatten()
                    .map(|file| file.file_name())
                    .collect();
                if files.contains(&OsString::from("thumbnail.jpg"))
                    && files.contains(&OsString::from("data.json"))
                {
                    self.cached_query(path, search, id)
                } else {
                    self.query(path, search, id).await
                }
            }
            Err(_) => self.query(path, search, id).await,
        }
    }

    async fn download_image(&self, url: &str) -> Result<reqwest::Response> {
        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::new(e.into(), url))?;
        Ok(res)
    }
}

impl Data {
    async fn download_image(mut self, api: &Api, path: &Path) -> Result<Data> {
        let bytes = api
            .download_image(&self.media.banner_image)
            .await?
            .bytes()
            .await
            .map_err(|e| Error::new(e.into(), &self.media.title.english))?;

        info!("Image downloaded for: {}", self.media.title.english);

        let name_file = path.join(".metadata").join("thumbnail.jpg");

        let mut file = std::fs::File::create(&name_file)
            .map_err(|e| Error::new(e.into(), &self.media.title.english))?;
        file.write_all(&bytes)
            .map_err(|e| Error::new(e.into(), &self.media.title.english))?;

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
        self.media.description = self
            .media
            .description
            .lines()
            .next()
            .unwrap()
            .replace("<br>", "");
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
