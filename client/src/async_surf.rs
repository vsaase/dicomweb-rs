use crate::{DICOMQueryBuilder, Error, Result};
use bytes::{Buf, Bytes};
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use dicomweb_util::{dicom_from_reader, json2dicom, parse_multipart_body};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{convert::TryInto, fmt::format, io::Cursor, marker::PhantomData};
use surf::Url;

use crate::DICOMWebClient;

impl From<surf::Error> for crate::Error {
    fn from(e: surf::Error) -> Self {
        crate::Error::Surf(e)
    }
}

#[derive(Default, Debug)]
pub struct DICOMWebClientSurf {
    client: surf::Client,
    config: surf::Config,
    url: Option<Url>,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMWebClient for DICOMWebClientSurf {
    type QueryBuilder = QueryBuilderSurf;

    fn default_headers(mut self, key: &'static str, value: &str) -> Self {
        self.config = self.config.add_header(key, value).unwrap();
        self
    }

    fn get_url(&mut self, url: &str) -> Self::QueryBuilder {
        let mut newurl = self.url.clone().unwrap();
        let path = format!("{}{}", newurl.path(), url);
        newurl.set_path(&path.as_str());
        QueryBuilderSurf {
            request_builder: self.client.get(newurl),
            query: Default::default(),
        }
    }

    fn get_qido_prefix(&self) -> &str {
        &self.qido_url_prefix
    }
    fn get_wado_prefix(&self) -> &str {
        &self.wado_url_prefix
    }
}

impl DICOMWebClientSurf {
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

pub struct QueryBuilderSurf {
    query: Query,
    request_builder: surf::RequestBuilder,
}

#[derive(Serialize, Deserialize, Default)]
struct Query {
    PatientName: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl DICOMQueryBuilder for QueryBuilderSurf {
    fn patient_name(mut self, name_query: &str) -> Self {
        self.query.PatientName = Some(name_query.to_string());
        self
    }

    fn limit(mut self, limit: u32) -> Self {
        self.query.limit = Some(limit);
        self
    }

    fn offset(mut self, offset: u32) -> Self {
        self.query.offset = Some(offset);
        self
    }
}

impl QueryBuilderSurf {
    pub async fn results(self) -> Result<Vec<InMemDicomObject>> {
        let mut res = self.request_builder.query(&self.query)?.send().await?;
        let content_type = res.header("content-type").unwrap().get(0).unwrap();
        println!("content-type: {}", content_type);

        if !content_type.as_str().starts_with("application/dicom+json") {
            return Err(Error::DICOMWeb(
                "invalid content type, should be application/dicom+json".to_string(),
            ));
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
