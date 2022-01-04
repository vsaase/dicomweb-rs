use std::convert::TryFrom;
use std::io::Cursor;

use crate::{Error, Result};
use bytes::Buf;
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use dicomweb_util::json2dicom;
use dicomweb_util::{dicom_from_reader, parse_multipart_body};
use http::header::HeaderName;
use http::{HeaderMap, HeaderValue};

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;

use serde::Serialize;
use serde_json::Value;

use super::{DICOMwebClientReqwest, QueryBuilderReqwest, RequestBuilderTrait};
use super::{ReqwestClient, ReqwestClientBuilder};

pub type Client = DICOMwebClientReqwest<reqwest::Client, reqwest::ClientBuilder>;

pub type QueryBuilder = QueryBuilderReqwest<reqwest::RequestBuilder>;

impl ReqwestClientBuilder for reqwest::ClientBuilder {
    type Client = reqwest::Client;

    fn new() -> Self {
        Self::new()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn proxy(self, proxy: Proxy) -> Self {
        self.proxy(proxy)
    }

    fn default_headers(self, headers: HeaderMap) -> Self {
        self.default_headers(headers)
    }
    fn build(self) -> reqwest::Result<Self::Client> {
        self.build()
    }
}

impl ReqwestClient for reqwest::Client {
    type ClientBuilder = reqwest::ClientBuilder;
    type RequestBuilder = reqwest::RequestBuilder;

    fn get<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder {
        self.get(url)
    }
    fn post<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder {
        self.post(url)
    }
}

impl RequestBuilderTrait for reqwest::RequestBuilder {
    fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        HeaderValue: TryFrom<V>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.header(key, value)
    }

    fn query<T: Serialize + ?Sized>(self, query: &T) -> Self {
        self.query(query)
    }

    fn body(self, body: Vec<u8>) -> Self {
        todo!()
    }
}

impl QueryBuilder {
    pub async fn results(self) -> Result<Vec<InMemDicomObject>> {
        let res = self.send().await?;
        let content_type = res.headers()["content-type"].to_str()?;
        println!("content-type: {}", content_type);

        if !content_type.starts_with("application/dicom+json") {
            return Err(Error::DICOMweb(
                "invalid content type, should be application/dicom+json".to_string(),
            ));
        }

        let json: Vec<Value> = res.json().await?;
        Ok(json2dicom(&json)?)
    }

    pub async fn dicoms(self) -> Result<Vec<DefaultDicomObject>> {
        let res = self.send().await?;
        let content_type = res.headers()["content-type"].to_str().unwrap();
        println!("content-type: {}", content_type);
        let (_, boundary) = content_type.rsplit_once("boundary=").unwrap();
        let boundary = String::from(boundary);
        println!("boundary: {}", boundary);

        let body = res.bytes().await?;
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

    pub async fn send(self) -> reqwest::Result<reqwest::Response> {
        self.request_builder.send().await
    }
}
