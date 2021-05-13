use dicom::core::Tag;
use dicom::object::DefaultDicomObject;
use http::{self, HeaderMap};
use reqwest;
use reqwest::header;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Proxy;
use serde::de::DeserializeOwned;
use serde_json::Value;

use bytes::Buf;
use error_chain::error_chain;
use std::convert::TryFrom;
use std::env;
use std::future::Future;
use std::{collections::HashMap, io::Cursor};
use util::{dicom_from_reader, parse_multipart_body};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        Serde(serde_json::Error);
        Dicom(dicom::object::Error);
        DicomCastValue(dicom::core::value::CastValueError);
    }

    errors{
        Custom(t: String) {
            description("custom")
            display("{}", t)
        }
    }
}

#[derive(Default)]
pub struct DICOMWebClientBuilder {
    client_builder: reqwest::ClientBuilder,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}
#[derive(Default)]
pub struct DICOMWebClient {
    client: reqwest::Client,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMWebClientBuilder {
    pub fn new(url: &str) -> DICOMWebClientBuilder {
        DICOMWebClientBuilder {
            client_builder: reqwest::Client::builder(),
            url: String::from(url),
            ..Default::default()
        }
    }

    pub fn proxy(mut self, proxy: Proxy) -> DICOMWebClientBuilder {
        self.client_builder = self.client_builder.proxy(proxy);
        self
    }

    pub fn build(self) -> reqwest::Result<DICOMWebClient> {
        let build = self.client_builder.build();
        if let Ok(client) = build {
            Ok(DICOMWebClient {
                client: client,
                url: self.url,
                qido_url_prefix: self.qido_url_prefix,
                wado_url_prefix: self.wado_url_prefix,
                stow_url_prefix: self.stow_url_prefix,
                ups_url_prefix: self.ups_url_prefix,
            })
        } else {
            Err(build.err().unwrap())
        }
    }

    pub fn default_headers(mut self, key: &'static str, value: &str) -> DICOMWebClientBuilder {
        let mut headers = header::HeaderMap::new();
        headers.insert(key, value.parse().unwrap());

        self.client_builder = self.client_builder.default_headers(headers);
        self
    }
}

impl DICOMWebClient {
    pub fn new(url: &str) -> DICOMWebClient {
        let mut builder = DICOMWebClientBuilder::new(url);
        if let Ok(proxy) = env::var("http_proxy") {
            builder = builder.proxy(reqwest::Proxy::http(proxy).unwrap());
        }
        builder.build().unwrap()
    }

    pub fn builder(url: &str) -> DICOMWebClientBuilder {
        DICOMWebClientBuilder::new(url)
    }

    pub fn find_studies(&self) -> QueryBuilder {
        let mut url = self.url.clone();
        url.push_str("/");
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies");
        QueryBuilder {
            client: &self,
            request_builder: self.client.get(&url),
        }
    }

    pub fn find_series(&self, study_instance_uid: &str) -> QueryBuilder {
        let mut url = self.url.clone();
        url.push_str("/");
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series");
        QueryBuilder {
            client: &self,
            request_builder: self.client.get(&url),
        }
    }

    pub fn find_instances(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> QueryBuilder {
        let mut url = self.url.clone();
        url.push_str("/");
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series/");
        url.push_str(series_instance_uid);
        url.push_str("/instances");
        QueryBuilder {
            client: &self,
            request_builder: self.client.get(&url),
        }
    }

    pub fn get_instance(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
        sop_instance_uid: &str,
    ) -> QueryBuilder {
        let mut url = self.url.clone();
        url.push_str("/");
        url.push_str(&self.wado_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series/");
        url.push_str(series_instance_uid);
        url.push_str("/instances/");
        url.push_str(sop_instance_uid);
        QueryBuilder {
            client: &self,
            request_builder: self.client.get(&url),
        }
    }
}

pub struct QueryBuilder<'a> {
    client: &'a DICOMWebClient,
    request_builder: reqwest::RequestBuilder,
}

impl<'a> QueryBuilder<'a> {
    pub fn header<K, V>(mut self, key: K, value: V) -> QueryBuilder<'a>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.request_builder = self.request_builder.header(key, value);
        self
    }

    pub fn patient_name(mut self, name_query: &'a str) -> QueryBuilder {
        self.request_builder = self.request_builder.query(&[("PatientName", name_query)]);
        self
    }

    pub fn limit(mut self, limit: u32) -> QueryBuilder<'a> {
        self.request_builder = self.request_builder.query(&[("limit", limit.to_string())]);
        self
    }

    pub async fn json<T: DeserializeOwned>(self) -> reqwest::Result<T> {
        let res = self.request_builder.send().await?;
        res.json().await
    }

    pub async fn dicoms(self) -> Result<Vec<DefaultDicomObject>> {
        let res = self.request_builder.send().await?;
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

    pub fn send(self) -> impl Future<Output = reqwest::Result<reqwest::Response>> {
        self.request_builder.send()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
