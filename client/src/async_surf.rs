use crate::Result;
use bytes::{Buf, Bytes};
use dicom::object::DefaultDicomObject;
use dicomweb_util::{dicom_from_reader, parse_multipart_body};
use serde::de::DeserializeOwned;
use std::{convert::TryInto, fmt::format, io::Cursor};
use surf::{Client, Config, Url};

use crate::DICOMWebClient;

#[derive(Default, Debug)]
pub struct DICOMWebClientSurf {
    client: Client,
    config: Config,
    url: Option<Url>,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMWebClientSurf {
    pub fn new(url: &str) -> Self {
        let config = Config::new();
        let client = Client::new();
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

    pub fn default_headers(mut self, key: &'static str, value: &str) -> Self {
        self.config = self.config.add_header(key, value).unwrap();
        self
    }

    pub fn build(mut self) -> Result<DICOMWebClientSurf> {
        self.client = self.config.clone().try_into().unwrap();
        Ok(self)
    }

    pub fn builder(url: &str) -> Self {
        Self::new(url)
    }

    pub fn find_studies(&self) -> QueryBuilderSurf {
        let mut url = self.url.clone().unwrap();
        let mut path: String = url.path().to_owned();
        path.push_str(&self.qido_url_prefix);
        path.push_str("/studies");
        url.set_path(&path.as_str());
        QueryBuilderSurf { url, client: &self }
    }

    pub fn find_series(&self, study_instance_uid: &str) -> QueryBuilderSurf {
        let mut url = self.url.clone().unwrap();
        let mut path: String = url.path().to_owned();
        path.push_str(&self.qido_url_prefix);
        path.push_str("/studies/");
        path.push_str(study_instance_uid);
        path.push_str("/series");
        url.set_path(&path.as_str());
        QueryBuilderSurf { url, client: &self }
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
        QueryBuilderSurf { url, client: &self }
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
        QueryBuilderSurf { url, client: &self }
    }
}

pub struct QueryBuilderSurf<'a> {
    url: Url,
    client: &'a DICOMWebClientSurf,
}

impl<'a> QueryBuilderSurf<'a> {
    pub fn patient_name(mut self, name_query: &str) -> Self {
        self.url
            .query_pairs_mut()
            .append_pair("PatientName", name_query);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.url
            .query_pairs_mut()
            .append_pair("limit", &format!("{}", limit));
        self
    }

    pub async fn json<T: DeserializeOwned>(&self) -> surf::Result<T> {
        self.client.client.get(self.url.as_str()).recv_json().await
    }

    pub async fn string(&self) -> surf::Result<String> {
        self.client
            .client
            .get(self.url.as_str())
            .recv_string()
            .await
    }

    pub async fn dicoms(self) -> Result<Vec<DefaultDicomObject>> {
        let mut res = self
            .client
            .client
            .get(self.url.as_str())
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
