use blocking::DICOMWebClientBlocking;
use dicom::core::Tag;
use dicom::object::DefaultDicomObject;
use http::{self, HeaderMap};
use reqwest;
use reqwest::header;
use reqwest::header::{HeaderName, HeaderValue};

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;

use serde::de::DeserializeOwned;
use serde_json::Value;

use bytes::Buf;
use dicomweb_util::{dicom_from_reader, json2dicom, parse_multipart_body};
use error_chain::error_chain;
use std::convert::TryFrom;
use std::env;
use std::future::Future;
use std::{collections::HashMap, io::Cursor};

pub use dicomweb_util::DICOMJson;

pub mod async_client;
pub mod blocking;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        Serde(serde_json::Error);
        Dicom(dicom::object::Error);
        DicomCastValue(dicom::core::value::CastValueError);
        Util(dicomweb_util::Error);
    }

    errors{
        Custom(t: String) {
            description("custom")
            display("{}", t)
        }
    }
}

pub trait ClientBuilderTrait {
    type WebClient;
    fn proxy(self, proxy: Proxy) -> Self;
    fn default_headers(self, headers: HeaderMap) -> Self;
}
impl ClientBuilderTrait for reqwest::blocking::ClientBuilder {
    type WebClient = reqwest::blocking::Client;
    fn proxy(self, proxy: Proxy) -> Self {
        self.proxy(proxy)
    }

    fn default_headers(self, headers: HeaderMap) -> Self {
        self.default_headers(headers)
    }
}
impl ClientBuilderTrait for reqwest::ClientBuilder {
    type WebClient = reqwest::Client;
    fn proxy(self, proxy: Proxy) -> Self {
        self.proxy(proxy)
    }

    fn default_headers(self, headers: HeaderMap) -> Self {
        self.default_headers(headers)
    }
}

#[derive(Default)]
pub struct DICOMWebClientBuilder<T: ClientBuilderTrait> {
    client_builder: T,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

// pub trait DICOMWebClientBuilderTrait {
//     fn new(url: &str) -> Self;

//     #[cfg(not(target_arch = "wasm32"))]
//     fn proxy(self, proxy: reqwest::Proxy) -> Self;
//     fn default_headers(self, key: &'static str, value: &str) -> Self;
// }

// impl<T: ClientBuilderTrait + Default> DICOMWebClientBuilderTrait for DICOMWebClientBuilder<T> {
impl<T: ClientBuilderTrait + Default> DICOMWebClientBuilder<T> {
    fn new(url: &str) -> Self
    where
        Self: Sized,
    {
        Self {
            url: String::from(url),
            ..Default::default()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
        self.client_builder = self.client_builder.proxy(proxy);
        self
    }

    pub fn default_headers(mut self, key: &'static str, value: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(key, value.parse().unwrap());

        self.client_builder = self.client_builder.default_headers(headers);
        self
    }
}

#[derive(Default)]
pub struct DICOMWebClient<T> {
    client: T,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

pub struct QueryBuilder<T> {
    request_builder: T,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
