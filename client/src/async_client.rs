use std::convert::TryFrom;
use std::env;
use std::io::Cursor;

use bytes::Buf;
use dicom::object::DefaultDicomObject;
use dicomweb_util::{dicom_from_reader, parse_multipart_body};
use http::header::{self, HeaderName};
use http::HeaderValue;
use serde::de::DeserializeOwned;

use crate::QueryBuilder;
use crate::Result;
use crate::{DICOMWebClient, DICOMWebClientBuilder};

pub type DICOMWebClientBuilderAsync = DICOMWebClientBuilder<reqwest::ClientBuilder>;
pub type DICOMWebClientAsync = DICOMWebClient<reqwest::Client>;

pub type QueryBuilderAsync = QueryBuilder<reqwest::RequestBuilder>;

impl DICOMWebClientBuilderAsync {
    pub fn build(self) -> reqwest::Result<DICOMWebClientAsync> {
        let build = self.client_builder.build();
        if let Ok(client) = build {
            Ok(DICOMWebClientAsync {
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
}

impl DICOMWebClientAsync {
    pub fn new(url: &str) -> DICOMWebClientAsync {
        let mut builder = DICOMWebClientBuilderAsync::new(url);

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(proxy) = env::var("http_proxy") {
            builder = builder.proxy(reqwest::Proxy::http(proxy).unwrap());
        }

        builder.build().unwrap()
    }

    pub fn builder(url: &str) -> DICOMWebClientBuilderAsync {
        DICOMWebClientBuilderAsync::new(url)
    }

    pub fn find_studies(&self) -> QueryBuilderAsync {
        let mut url = self.url.clone();
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies");
        QueryBuilder {
            request_builder: self.client.get(&url),
        }
    }

    pub fn find_series(&self, study_instance_uid: &str) -> QueryBuilderAsync {
        let mut url = self.url.clone();
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series");
        QueryBuilder {
            request_builder: self.client.get(&url),
        }
    }

    pub fn find_instances(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> QueryBuilderAsync {
        let mut url = self.url.clone();
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series/");
        url.push_str(series_instance_uid);
        url.push_str("/instances");
        QueryBuilder {
            request_builder: self.client.get(&url),
        }
    }

    pub fn get_instance(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
        sop_instance_uid: &str,
    ) -> QueryBuilderAsync {
        let mut url = self.url.clone();
        url.push_str(&self.wado_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series/");
        url.push_str(series_instance_uid);
        url.push_str("/instances/");
        url.push_str(sop_instance_uid);
        QueryBuilder {
            request_builder: self.client.get(&url),
        }
    }
}

impl QueryBuilderAsync {
    pub fn header<K, V>(mut self, key: K, value: V) -> QueryBuilderAsync
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.request_builder = self.request_builder.header(key, value);
        self
    }

    pub fn patient_name(mut self, name_query: &str) -> QueryBuilderAsync {
        self.request_builder = self.request_builder.query(&[("PatientName", name_query)]);
        self
    }

    pub fn limit(mut self, limit: u32) -> QueryBuilderAsync {
        self.request_builder = self.request_builder.query(&[("limit", limit.to_string())]);
        self
    }

    pub fn offset(mut self, offset: u32) -> QueryBuilderAsync {
        self.request_builder = self
            .request_builder
            .query(&[("offset", offset.to_string())]);
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

    pub async fn send(self) -> reqwest::Result<reqwest::Response> {
        self.request_builder.send().await
    }
}
