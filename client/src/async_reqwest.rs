use std::convert::TryFrom;
use std::io::Cursor;

use bytes::Buf;
use dicom::object::DefaultDicomObject;
use dicomweb_util::{dicom_from_reader, parse_multipart_body};
use http::header::HeaderName;
use http::{HeaderMap, HeaderValue};
use reqwest::Proxy;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::DICOMWebClientReqwest;
use crate::{QueryBuilderReqwest, RequestBuilderTrait};
use crate::{ReqwestClient, ReqwestClientBuilder, Result};

pub type DICOMWebClientAsync = DICOMWebClientReqwest<reqwest::Client, reqwest::ClientBuilder>;

pub type QueryBuilderAsync = QueryBuilderReqwest<reqwest::RequestBuilder>;

impl ReqwestClientBuilder for reqwest::ClientBuilder {
    type Client = reqwest::Client;

    fn new() -> Self {
        Self::new()
    }

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
}

impl QueryBuilderAsync {
    pub async fn json<T: DeserializeOwned>(self) -> reqwest::Result<T> {
        let res = self.send().await?;
        res.json().await
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
