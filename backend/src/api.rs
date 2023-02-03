use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
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
    coverImage {
        large,
    },
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    id: usize,
    pub title: Title,
    pub description: String,
    pub genres: Vec<String>,
    cover_image: CoverImage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Title {
    pub romaji: String,
    pub english: String,
    pub native: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CoverImage {
    #[serde(rename = "large")]
    image: String,
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

    pub async fn query(&self, path: &Path, search: &str, id: usize) -> Result<Data> {
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

        let data = result.data.download_image(self, path);
        let mut data = data.await?;
        data.set_id(id);
        Ok(data)
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
            .download_image(&self.media.cover_image.image)
            .await?
            .bytes()
            .await
            .map_err(|e| Error::new(e.into(), &self.media.title.english))?;

        info!("Image downloaded for: {}", self.media.title.english);

        let name_file = path.join("thumbnail.png");

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
}
