use std::convert::TryFrom;
use std::io::Cursor;

use crate::{Error, Result};
use bytes::Buf;
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use dicomweb_util::{dicom_from_reader, json2dicom, parse_multipart_body};
use http::header::HeaderName;
use http::{HeaderMap, HeaderValue};

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;

use serde::Serialize;
use serde_json::Value;

use super::RequestBuilderTrait;
use super::{DICOMwebClientReqwest, QueryBuilderReqwest, ReqwestClient, ReqwestClientBuilder};

pub type Client =
    DICOMwebClientReqwest<reqwest::blocking::Client, reqwest::blocking::ClientBuilder>;

pub type QueryBuilder = QueryBuilderReqwest<reqwest::blocking::RequestBuilder>;

impl ReqwestClientBuilder for reqwest::blocking::ClientBuilder {
    type Client = reqwest::blocking::Client;

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

impl ReqwestClient for reqwest::blocking::Client {
    type ClientBuilder = reqwest::blocking::ClientBuilder;
    type RequestBuilder = reqwest::blocking::RequestBuilder;

    fn get<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder {
        self.get(url)
    }
    fn post<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder {
        self.post(url)
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

    fn body(self, body: Vec<u8>) -> Self {
        self.body(body)
    }
}

impl QueryBuilder {
    pub fn results(self) -> Result<Vec<InMemDicomObject>> {
        let res = self.send()?;
        let content_type = res
            .headers()
            .get("content-type")
            .ok_or(Error::DICOMweb(
                "no content type on response, should be application/dicom+json".to_string(),
            ))?
            .to_str()?;
        println!("content-type: {}", content_type);

        if !content_type.starts_with("application/dicom+json") {
            return Err(Error::DICOMweb(
                "invalid content type, should be application/dicom+json".to_string(),
            ));
        }

        let json: Vec<Value> = res.json()?;
        Ok(json2dicom(&json)?)
    }

    pub fn dicoms(self) -> Result<Vec<DefaultDicomObject>> {
        let res = self.send()?;
        let content_type = res
            .headers()
            .get("content-type")
            .ok_or(Error::DICOMweb(
                "no content type on response, should be multipart/related".to_string(),
            ))?
            .to_str()?;
        println!("content-type: {}", content_type);
        if !content_type.starts_with("multipart/related") {
            return Err(Error::DICOMweb(
                "invalid content type, should be multipart/related".to_string(),
            ));
        }
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
