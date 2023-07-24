pub mod query;

pub use query::*;

use crate::backend::title::Title;
use crate::Result;

use anyhow::bail;
use hyper::{body::Buf, client::HttpConnector, Body, Client, Method, Request, Response};
use hyper_tls::HttpsConnector;
use serde_json::json;
use std::{ffi::OsString, path::Path};

/// [`Client`] connected to the Anilist API.
#[derive(Debug)]
pub struct Anilist {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl Default for Anilist {
    fn default() -> Anilist {
        Anilist::new()
    }
}

impl Anilist {
    /// New [`Client`] connected with a [`HttpsConnector`].
    pub fn new() -> Anilist {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        Anilist { client }
    }

    /// POST the [`QUERY`] to the Anilist API with the title name as it's variable.
    ///
    /// Downloads a json-file and a jpg-file.
    async fn query(&self, path: &Path, title_search: &str, id: usize) -> Result<Data> {
        let json = json!({"query": QUERY, "variables": {"search": title_search}});

        let req = Request::builder()
            .method(Method::POST)
            .uri("https://graphql.anilist.co/")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Body::from(json.to_string()))?;

        let resp = self.client.request(req).await?;
        let body = hyper::body::aggregate(resp).await?;
        let result: Query = serde_json::from_reader(body.reader())?;

        let content = serde_json::to_string_pretty(&result)?;
        std::fs::write(path.join(".metadata").join("data.json"), content)?;

        let mut data = result.data.download_image(self, path).await?;
        data.set_id(id);
        data.find_studio();
        data.clean_description();

        Ok(data)
    }

    /// Grabs the json-file and a jpg-file from a previously made [`Query`].
    fn cached_query(&self, path: &Path, id: usize) -> Result<Data> {
        let content = std::fs::read_to_string(path.join(".metadata").join("data.json"))?;

        let result: Query = serde_json::from_str(&content)?;

        let mut data = result.data;
        data.set_id(id);
        data.find_studio();
        data.clean_description();
        data.set_thumbnail_path(path.join(".metadata").join("thumbnail.jpg"));

        Ok(data)
    }

    /// GET Request of the indicated url.
    pub async fn get_body(&self, url: &str) -> Result<Response<Body>> {
        let uri: hyper::Uri = url.parse()?;
        let res = self.client.get(uri).await?;
        Ok(res)
    }

    /// Checks if a [`Query`] was previously made for this [`Title`] or makes a new one.
    pub async fn try_query(&self, title: &mut Title, id: usize) -> Result<()> {
        let path = title.path.as_path();
        let search = &title.name;

        if let Ok(files) = std::fs::read_dir(path.join(".metadata")) {
            let files: Vec<_> = files
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
}
