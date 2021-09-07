use serde::de::DeserializeOwned;
use std::{convert::TryInto, fmt::format};
use surf::{Client, Config, Url};

use crate::DICOMWebClient;

#[derive(Default, Debug)]
pub struct DICOMWebClientSurf {
    client: Client,
    config: Config,
    pub url: Option<Url>,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMWebClientSurf {
    pub fn new(url: &str) -> Self {
        let url = url.to_owned() + "/";
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

    pub fn build(mut self) -> Result<DICOMWebClientSurf, ()> {
        self.client = self.config.clone().try_into().unwrap();
        Ok(self)
    }

    pub fn builder(url: &str) -> Self {
        Self::new(url)
    }

    pub fn find_studies(mut self) -> Self {
        if let Some(ref mut url) = self.url {
            let mut path: String = url.path().to_owned();
            path.push_str(&self.qido_url_prefix);
            path.push_str("/studies");
            url.set_path(&path.as_str());
        };
        self
    }

    pub fn find_series(mut self, study_instance_uid: &str) -> Self {
        if let Some(ref mut url) = self.url {
            let mut path: String = url.path().to_owned();
            path.push_str(&self.qido_url_prefix);
            path.push_str("/studies");
            path.push_str(study_instance_uid);
            path.push_str("/series");
            url.set_path(&path.as_str());
        };
        self
    }

    pub fn patient_name(mut self, name_query: &str) -> Self {
        if let Some(ref mut url) = self.url {
            url.query_pairs_mut().append_pair("PatientName", name_query);
        }
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        if let Some(ref mut url) = self.url {
            url.query_pairs_mut()
                .append_pair("limit", &format!("{}", limit));
        }
        self
    }

    pub async fn json<T: DeserializeOwned>(&self) -> surf::Result<T> {
        self.client
            .get(self.url.as_ref().unwrap().as_str())
            .recv_json()
            .await
    }

    pub async fn string(&self) -> surf::Result<String> {
        self.client
            .get(self.url.as_ref().unwrap().as_str())
            .recv_string()
            .await
    }
}
