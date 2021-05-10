use dicom::core::Tag;
use http;
use reqwest;
use reqwest::header::{HeaderName, HeaderValue};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::future::Future;

pub type Error = reqwest::Error;
#[derive(Default)]
pub struct DICOMWebClient {
    client: reqwest::Client,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMWebClient {
    pub fn new(url: &str) -> DICOMWebClient {
        DICOMWebClient {
            client: reqwest::Client::builder()
                .proxy(reqwest::Proxy::http(env::var("http_proxy").unwrap()).unwrap())
                .build()
                .unwrap(),
            url: String::from(url),
            ..Default::default()
        }
    }

    pub fn find_studies(&self) -> QueryBuilder {
        let mut url = self.url.clone();
        url.push_str("/studies");
        QueryBuilder {
            client: &self,
            request_builder: self.client.get(&url),
        }
    }

    pub fn find_series(&self, study_instance_uid: &str) -> QueryBuilder {
        let mut url = self.url.clone();
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

    pub fn send(self) -> impl Future<Output = Result<reqwest::Response, reqwest::Error>> {
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
