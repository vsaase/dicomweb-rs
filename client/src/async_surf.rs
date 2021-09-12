use crate::Result;
use bytes::{Buf, Bytes};
use dicom::object::DefaultDicomObject;
use dicomweb_util::{dicom_from_reader, parse_multipart_body};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt::format, io::Cursor, marker::PhantomData};
use surf::Url;

use crate::DICOMWebClient;

#[derive(Default, Debug)]
pub struct DICOMWebClientSurf {
    client: surf::Client,
    config: surf::Config,
    url: Option<Url>,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMWebClient for DICOMWebClientSurf {
    type Client = surf::Client;
    type Config = surf::Config;
    type QueryBuilder = QueryBuilderSurf;

    fn default_headers(mut self, key: &'static str, value: &str) -> Self {
        self.config = self.config.add_header(key, value).unwrap();
        self
    }

    fn find_studies(&mut self) -> Self::QueryBuilder {
        let mut url = self.url.clone().unwrap();
        let mut path: String = url.path().to_owned();
        path.push_str(&self.qido_url_prefix);
        path.push_str("/studies");
        url.set_path(&path.as_str());
        QueryBuilderSurf {
            request_builder: self.client.get(url),
            query: Default::default(),
        }
    }
}

impl DICOMWebClientSurf {
    pub fn new(url: &str) -> Self {
        let config = surf::Config::new();
        let client = surf::Client::new();
        Self {
            client,
            config,
            url: Some(Url::parse(&url).unwrap()),
            ..Default::default()
        }
    }

    // #[cfg(not(target_arch = "wasm32"))]
    // fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
    //     self.client_builder = self.client_builder.proxy(proxy);
    //     self
    // }

    pub fn build(mut self) -> Result<DICOMWebClientSurf> {
        self.client = self.config.clone().try_into().unwrap();
        Ok(self)
    }

    pub fn builder(url: &str) -> Self {
        Self::new(url)
    }

    pub fn find_series(&self, study_instance_uid: &str) -> QueryBuilderSurf {
        let mut url = self.url.clone().unwrap();
        let mut path: String = url.path().to_owned();
        path.push_str(&self.qido_url_prefix);
        path.push_str("/studies/");
        path.push_str(study_instance_uid);
        path.push_str("/series");
        url.set_path(&path.as_str());
        QueryBuilderSurf {
            request_builder: self.client.get(url),
            query: Default::default(),
        }
    }

    pub fn find_instances(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> QueryBuilderSurf {
        let mut url = self.url.clone().unwrap();
        let mut path: String = url.path().to_owned();
        path.push_str(&self.qido_url_prefix);
        path.push_str("/studies/");
        path.push_str(study_instance_uid);
        path.push_str("/series/");
        path.push_str(series_instance_uid);
        path.push_str("/instances");
        url.set_path(&path.as_str());
        QueryBuilderSurf {
            request_builder: self.client.get(url),
            query: Default::default(),
        }
    }

    pub fn get_instance(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
        sop_instance_uid: &str,
    ) -> QueryBuilderSurf {
        let mut url = self.url.clone().unwrap();
        let mut path: String = url.path().to_owned();
        path.push_str(&self.wado_url_prefix);
        path.push_str("/studies/");
        path.push_str(study_instance_uid);
        path.push_str("/series/");
        path.push_str(series_instance_uid);
        path.push_str("/instances/");
        path.push_str(sop_instance_uid);
        url.set_path(&path.as_str());
        QueryBuilderSurf {
            request_builder: self.client.get(url),
            query: Default::default(),
        }
    }
}

pub struct QueryBuilderSurf {
    query: Query,
    request_builder: surf::RequestBuilder,
}

#[derive(Serialize, Deserialize, Default)]
struct Query {
    PatientName: Option<String>,
    limit: Option<u32>,
}

impl QueryBuilderSurf {
    pub fn patient_name(mut self, name_query: &str) -> Self {
        self.query.PatientName = Some(name_query.to_string());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.query.limit = Some(limit);
        self
    }

    pub async fn json<T: DeserializeOwned>(self) -> surf::Result<T> {
        self.request_builder.query(&self.query)?.recv_json().await
    }

    pub async fn string(self) -> surf::Result<String> {
        self.request_builder.query(&self.query)?.recv_string().await
    }

    pub async fn dicoms(self) -> Result<Vec<DefaultDicomObject>> {
        let mut res = self
            .request_builder
            .query(&self.query)
            .unwrap()
            .send()
            .await
            .unwrap();
        let content_type = res.header("content-type").unwrap().get(0).unwrap();
        println!("content-type: {}", content_type);
        let (_, boundary) = content_type.as_str().rsplit_once("boundary=").unwrap();
        let boundary = String::from(boundary);
        println!("boundary: {}", boundary);

        let body: Bytes = res.body_bytes().await.unwrap().into();
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
}
