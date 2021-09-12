use http::{self, HeaderMap};
use reqwest;
use reqwest::header;
use reqwest::header::{HeaderName, HeaderValue};

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;

use serde::Serialize;

use error_chain::error_chain;
use std::convert::TryFrom;
use std::env;

pub use dicomweb_util::DICOMJson;

pub mod async_reqwest;
pub mod async_surf;
pub mod blocking_reqwest;

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

pub trait ReqwestClientBuilder {
    type Client: ReqwestClient + Default;

    fn new() -> Self;
    fn proxy(self, proxy: Proxy) -> Self;
    fn default_headers(self, headers: HeaderMap) -> Self;
    fn build(self) -> reqwest::Result<Self::Client>;
}

pub trait ReqwestClient {
    type ClientBuilder: ReqwestClientBuilder + Default;
    type RequestBuilder;

    fn get<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder;
}

pub trait DICOMWebClient {
    type Client;
    type Config;
    type QueryBuilder;

    fn default_headers(self, key: &'static str, value: &str) -> Self;
    fn find_studies(&mut self) -> Self::QueryBuilder;
}

#[derive(Default)]
pub struct DICOMWebClientReqwest<C, B> {
    client: Option<C>,
    config: Option<B>,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl<C: ReqwestClient, B: ReqwestClientBuilder<Client = C>> DICOMWebClient for DICOMWebClientReqwest<C, B> {
    type Client = C;
    type Config = B;
    type QueryBuilder = QueryBuilderReqwest<C::RequestBuilder>;

    fn default_headers(mut self, key: &'static str, value: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(key, value.parse().unwrap());

        if let Some(client_builder) = self.config.take() {
            self.config = Some(client_builder.default_headers(headers));
        }
        self
    }

    fn find_studies(&mut self) -> QueryBuilderReqwest<C::RequestBuilder> {
        self.make_client();
        let mut url = format!("{}{}/studies", self.url, self.qido_url_prefix);
        QueryBuilderReqwest {
            request_builder: self.client.as_ref().unwrap().get(&url),
        }
    }
}

impl<C: ReqwestClient, B: ReqwestClientBuilder<Client = C>> DICOMWebClientReqwest<C, B> {
    #[cfg(not(target_arch = "wasm32"))]
    fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
        if let Some(client_builder) = self.config.take() {
            self.config = Some(client_builder.proxy(proxy));
        }
        self
    }

    fn make_client(&mut self) {
        if let Some(client_builder) = self.config.take() {
            self.client = client_builder.build().ok();
        }
    }


    pub fn new(url: &str) -> Self {
        let client_builder = Some(B::new());
        let mut dicomwebclient = Self {
            client: None,
            config: client_builder,
            url: String::from(url),
            qido_url_prefix: String::default(),
            wado_url_prefix: String::default(),
            stow_url_prefix: String::default(),
            ups_url_prefix: String::default(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(proxy) = env::var("http_proxy") {
            dicomwebclient = dicomwebclient.proxy(reqwest::Proxy::http(proxy).unwrap());
        }

        dicomwebclient
    }


    pub fn find_series(&mut self, study_instance_uid: &str) -> QueryBuilderReqwest<C::RequestBuilder> {
        self.make_client();
        let mut url = self.url.clone();
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series");
        QueryBuilderReqwest {
            request_builder: self.client.as_ref().unwrap().get(&url),
        }
    }

    pub fn find_instances(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> QueryBuilderReqwest<C::RequestBuilder> {
        let mut url = self.url.clone();
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series/");
        url.push_str(series_instance_uid);
        url.push_str("/instances");
        QueryBuilderReqwest {
            request_builder: self.client.as_ref().unwrap().get(&url),
        }
    }

    pub fn get_instance(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
        sop_instance_uid: &str,
    ) -> QueryBuilderReqwest<C::RequestBuilder> {
        let mut url = self.url.clone();
        url.push_str(&self.wado_url_prefix);
        url.push_str("/studies/");
        url.push_str(study_instance_uid);
        url.push_str("/series/");
        url.push_str(series_instance_uid);
        url.push_str("/instances/");
        url.push_str(sop_instance_uid);
        QueryBuilderReqwest {
            request_builder: self.client.as_ref().unwrap().get(&url),
        }
    }
}

pub struct QueryBuilderReqwest<T> {
    request_builder: T,
}

pub trait RequestBuilderTrait {
    fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        HeaderValue: TryFrom<V>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>;
    fn query<T: Serialize + ?Sized>(self, query: &T) -> Self;
}

impl<T: RequestBuilderTrait> QueryBuilderReqwest<T> {
    pub fn header<K, V>(mut self, key: K, value: V) -> QueryBuilderReqwest<T>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.request_builder = self.request_builder.header(key, value);
        self
    }

    pub fn patient_name(mut self, name_query: &str) -> QueryBuilderReqwest<T> {
        self.request_builder = self.request_builder.query(&[("PatientName", name_query)]);
        self
    }

    pub fn limit(mut self, limit: u32) -> QueryBuilderReqwest<T> {
        self.request_builder = self.request_builder.query(&[("limit", limit.to_string())]);
        self
    }

    pub fn offset(mut self, offset: u32) -> QueryBuilderReqwest<T> {
        self.request_builder = self
            .request_builder
            .query(&[("offset", offset.to_string())]);
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
