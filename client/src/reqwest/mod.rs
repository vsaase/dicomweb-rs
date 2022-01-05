use reqwest;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub use reqwest::Error;
#[cfg(not(target_arch = "wasm32"))]
pub use reqwest::Proxy;
use serde::Serialize;
use std::convert::TryFrom;
use std::env;

use crate::{DICOMQueryBuilder, DICOMwebClient};

pub mod async_reqwest;

#[cfg(feature = "blocking")]
pub mod blocking_reqwest;

pub trait ReqwestClientBuilder {
    type Client: ReqwestClient + Default;

    fn new() -> Self;
    #[cfg(not(target_arch = "wasm32"))]
    fn proxy(self, proxy: Proxy) -> Self;
    fn default_headers(self, headers: HeaderMap) -> Self;
    fn build(self) -> reqwest::Result<Self::Client>;
}

pub trait ReqwestClient {
    type ClientBuilder: ReqwestClientBuilder + Default;
    type RequestBuilder: RequestBuilderTrait;

    fn get<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder;
    fn post<U: reqwest::IntoUrl>(&self, url: U) -> Self::RequestBuilder;
}

#[derive(Default)]
pub struct DICOMwebClientReqwest<C, B> {
    client: Option<C>,
    config: Option<B>,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    _ups_url_prefix: String,
    boundary: String,
}

impl<C: ReqwestClient, B: ReqwestClientBuilder<Client = C>> DICOMwebClient
    for DICOMwebClientReqwest<C, B>
{
    type QueryBuilder = QueryBuilderReqwest<C::RequestBuilder>;

    fn default_headers(mut self, key: &'static str, value: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(key, value.parse().unwrap());

        if let Some(client_builder) = self.config.take() {
            self.config = Some(client_builder.default_headers(headers));
        }
        self
    }

    fn get_url(&mut self, url: &str) -> Self::QueryBuilder {
        self.make_client();
        let url = format!("{}{}", self.url, url);
        QueryBuilderReqwest {
            request_builder: self.client.as_ref().unwrap().get(url),
            boundary: self.get_boundary(),
        }
    }

    fn post_url(&mut self, url: &str) -> Self::QueryBuilder {
        self.make_client();
        let url = format!("{}{}", self.url, url);
        QueryBuilderReqwest {
            request_builder: self.client.as_ref().unwrap().post(url),
            boundary: self.get_boundary(),
        }
    }

    fn get_qido_prefix(&self) -> &str {
        &self.qido_url_prefix
    }
    fn get_wado_prefix(&self) -> &str {
        &self.wado_url_prefix
    }
    fn get_stow_prefix(&self) -> &str {
        &self.stow_url_prefix
    }

    fn set_boundary(&mut self, boundary: &str) {
        self.boundary = boundary.to_string();
    }

    fn get_boundary(&self) -> String {
        self.boundary.clone()
    }
}

impl<C: ReqwestClient, B: ReqwestClientBuilder<Client = C>> DICOMwebClientReqwest<C, B> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
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
            _ups_url_prefix: String::default(),
            boundary: String::default(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(proxy) = env::var("http_proxy") {
            dicomwebclient = dicomwebclient.proxy(reqwest::Proxy::http(proxy).unwrap());
        }

        dicomwebclient
    }
}

pub struct QueryBuilderReqwest<T> {
    request_builder: T,
    boundary: String,
}

pub trait RequestBuilderTrait {
    fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        HeaderValue: TryFrom<V>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>;
    fn query<T: Serialize + ?Sized>(self, query: &T) -> Self;
    fn body(self, body: Vec<u8>) -> Self;
}

impl<T: RequestBuilderTrait> DICOMQueryBuilder for QueryBuilderReqwest<T> {
    fn query(mut self, key: &str, value: &str) -> Self {
        self.request_builder = self.request_builder.query(&[(key, value)]);
        self
    }

    fn header(mut self, key: &str, value: &str) -> Self {
        self.request_builder = self.request_builder.header(key, value);
        self
    }

    fn body(mut self, body: Vec<u8>) -> Self {
        self.request_builder = self.request_builder.body(body);
        self
    }

    fn get_boundary(&self) -> String {
        self.boundary.clone()
    }
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
}
