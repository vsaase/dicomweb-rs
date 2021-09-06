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
    type Client: ReqwestClient;
    fn proxy(self, proxy: Proxy) -> Self;
    fn default_headers(self, headers: HeaderMap) -> Self;
    fn build(self) -> reqwest::Result<Self::Client>;
}

#[derive(Default)]
pub struct DICOMWebClientBuilder<T> {
    client_builder: T,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl<T: ReqwestClientBuilder + Default> DICOMWebClientBuilder<T> {
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

    pub fn build(self) -> reqwest::Result<DICOMWebClient<T::Client>> {
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
}

pub trait ReqwestClient {
    type ClientBuilder: ReqwestClientBuilder + Default;
    type RequestBuilder;

    fn get<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder;
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

impl<T: ReqwestClient> DICOMWebClient<T> {
    pub fn new(
        url: &str,
    ) -> DICOMWebClient<<<T as ReqwestClient>::ClientBuilder as ReqwestClientBuilder>::Client> {
        let mut builder = DICOMWebClientBuilder::<T::ClientBuilder>::new(url);

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(proxy) = env::var("http_proxy") {
            builder = builder.proxy(reqwest::Proxy::http(proxy).unwrap());
        }

        builder.build().unwrap()
    }

    pub fn builder(url: &str) -> DICOMWebClientBuilder<T::ClientBuilder> {
        DICOMWebClientBuilder::<T::ClientBuilder>::new(url)
    }

    pub fn find_studies(&self) -> QueryBuilder<T::RequestBuilder> {
        let mut url = self.url.clone();
        url.push_str(&self.qido_url_prefix);
        url.push_str("/studies");
        QueryBuilder {
            request_builder: self.client.get(&url),
        }
    }

    pub fn find_series(&self, study_instance_uid: &str) -> QueryBuilder<T::RequestBuilder> {
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
    ) -> QueryBuilder<T::RequestBuilder> {
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
    ) -> QueryBuilder<T::RequestBuilder> {
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

pub struct QueryBuilder<T> {
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

impl<T: RequestBuilderTrait> QueryBuilder<T> {
    pub fn header<K, V>(mut self, key: K, value: V) -> QueryBuilder<T>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.request_builder = self.request_builder.header(key, value);
        self
    }

    pub fn patient_name(mut self, name_query: &str) -> QueryBuilder<T> {
        self.request_builder = self.request_builder.query(&[("PatientName", name_query)]);
        self
    }

    pub fn limit(mut self, limit: u32) -> QueryBuilder<T> {
        self.request_builder = self.request_builder.query(&[("limit", limit.to_string())]);
        self
    }

    pub fn offset(mut self, offset: u32) -> QueryBuilder<T> {
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
