use std::convert::TryFrom;
use std::io::Cursor;

use crate::Result;
use bytes::Buf;
use dicom::object::DefaultDicomObject;
use dicomweb_util::{dicom_from_reader, parse_multipart_body};
use http::header::HeaderName;
use http::{HeaderMap, HeaderValue};
use reqwest::Proxy;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::RequestBuilderTrait;
use super::{DICOMWebClientReqwest, QueryBuilderReqwest, ReqwestClient, ReqwestClientBuilder};

pub type DICOMWebClientBlocking =
    DICOMWebClientReqwest<reqwest::blocking::Client, reqwest::blocking::ClientBuilder>;

pub type QueryBuilderBlocking = QueryBuilderReqwest<reqwest::blocking::RequestBuilder>;

impl ReqwestClientBuilder for reqwest::blocking::ClientBuilder {
    type Client = reqwest::blocking::Client;

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

impl ReqwestClient for reqwest::blocking::Client {
    type ClientBuilder = reqwest::blocking::ClientBuilder;
    type RequestBuilder = reqwest::blocking::RequestBuilder;

    fn get<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder {
        self.get(url)
    }
}

impl RequestBuilderTrait for reqwest::blocking::RequestBuilder {
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

impl QueryBuilderBlocking {
    pub fn json<T: DeserializeOwned>(self) -> reqwest::Result<T> {
        let res = self.send()?;
        res.json()
    }

    pub fn dicoms(self) -> Result<Vec<DefaultDicomObject>> {
        let res = self.send()?;
        let content_type = res.headers()["content-type"].to_str().unwrap();
        println!("content-type: {}", content_type);
        let (_, boundary) = content_type.rsplit_once("boundary=").unwrap();
        let boundary = String::from(boundary);
        println!("boundary: {}", boundary);

        let body = res.bytes()?;
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

    pub fn send(self) -> reqwest::Result<reqwest::blocking::Response> {
        self.request_builder.send()
    }
}
