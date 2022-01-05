use crate::{DICOMQueryBuilder, Result};
use bytes::{Buf, Bytes};
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use dicomweb_util::{dicom_from_reader, json2dicom, parse_multipart_body};
use log::debug;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;
use surf::Url;

use crate::DICOMwebClient;

impl From<surf::Error> for crate::Error {
    fn from(e: surf::Error) -> Self {
        crate::Error::Surf(e)
    }
}

#[derive(Default, Debug)]
pub struct Client {
    client: surf::Client,
    config: surf::Config,
    url: Option<Url>,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMwebClient for Client {
    type QueryBuilder = QueryBuilder;

    fn default_headers(mut self, key: &'static str, value: &str) -> Self {
        self.config = self.config.add_header(key, value).unwrap();
        self
    }

    fn get_url(&mut self, url: &str) -> Self::QueryBuilder {
        let mut newurl = self.url.clone().unwrap();
        let mut basepath = newurl.path();
        if basepath == "/" {
            basepath = "";
        }
        let path = format!("{}{}", basepath, url);
        newurl.set_path(&path.as_str());
        QueryBuilder {
            request_builder: self.client.get(newurl),
            query: Default::default(),
        }
    }

    fn post_url(&mut self, url: &str) -> Self::QueryBuilder {
        let mut newurl = self.url.clone().unwrap();
        let mut basepath = newurl.path();
        if basepath == "/" {
            basepath = "";
        }
        let path = format!("{}{}", basepath, url);
        newurl.set_path(&path.as_str());
        QueryBuilder {
            request_builder: self.client.post(newurl),
            query: Default::default(),
        }
    }

    fn get_qido_prefix(&self) -> &str {
        &self.qido_url_prefix
    }
    fn get_wado_prefix(&self) -> &str {
        &self.wado_url_prefix
    }
    fn get_stow_prefix(&self) -> &str {
        &self.stow_url_prefix
    }

    fn set_boundary(&mut self, boundary: &str) {
        todo!()
    }

    fn get_boundary(&self) -> String {
        todo!()
    }
}

impl Client {
    pub fn new(url: &str) -> Self {
        let config = surf::Config::new();
        let client = surf::Client::new();
        Self {
            client,
            config,
            url: Some(Url::parse(&url).unwrap()),
            ..Default::default()
        }
    }

    // #[cfg(not(target_arch = "wasm32"))]
    // fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
    //     self.client_builder = self.client_builder.proxy(proxy);
    //     self
    // }
}

pub struct QueryBuilder {
    query: HashMap<String, String>,
    request_builder: surf::RequestBuilder,
}

impl DICOMQueryBuilder for QueryBuilder {
    fn query(mut self, key: &str, value: &str) -> Self {
        self.query.insert(key.to_string(), value.to_string());
        self
    }

    fn header(mut self, key: &str, value: &str) -> Self {
        self.request_builder = self.request_builder.header(key, value);
        self
    }

    fn body(self, body: Vec<u8>) -> Self {
        todo!()
    }

    fn get_boundary(&self) -> String {
        todo!()
    }
}

impl QueryBuilder {
    pub async fn results(self) -> Result<Vec<InMemDicomObject>> {
        let req = self.request_builder.query(&self.query)?;
        debug!("req: {:?}", req);
        let mut res = req.send().await?;
        let content_type = res.header("content-type").unwrap().get(0).unwrap();
        println!("content-type: {}", content_type);

        if !content_type.as_str().starts_with("application/dicom+json") {
            panic!(
                "invalid content type, should be application/dicom+json,  response: {:?}",
                res
            )
            // return Err(Error::DICOMweb(
            //     "invalid content type, should be application/dicom+json".to_string(),
            // ));
        }

        let json: Vec<Value> = res.body_json().await?;
        Ok(json2dicom(&json)?)
    }

    pub async fn dicoms(self) -> Result<Vec<DefaultDicomObject>> {
        let mut res = self.request_builder.query(&self.query)?.send().await?;
        let content_type = res.header("content-type").unwrap().get(0).unwrap();
        println!("content-type: {}", content_type);
        let (_, boundary) = content_type.as_str().rsplit_once("boundary=").unwrap();
        let boundary = String::from(boundary);
        println!("boundary: {}", boundary);

        let body: Bytes = res.body_bytes().await?.into();
        let parts = parse_multipart_body(body, &boundary)?;
        let result = parts
            .iter()
            .map(|part| {
                let reader = Cursor::new(part).reader();
                dicom_from_reader(reader).unwrap()
            })
            .collect();
        Ok(result)
    }
}
