use dicom::object::InMemDicomObject;
use log::info;
use thiserror::Error;

pub use dicomweb_util::DICOMJson;
pub mod async_surf;
pub mod reqwest;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Surf(surf::Error),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Dicom(#[from] dicom::object::Error),
    #[error("{0}")]
    DicomCastValue(#[from] dicom::core::value::CastValueError),
    #[error("{0}")]
    Util(#[from] dicomweb_util::Error),
    #[error("{0}")]
    Http(#[from] http::header::ToStrError),
    #[error("{0}")]
    DICOMweb(String),
}
pub type Result<T> = std::result::Result<T, Error>;

pub trait DICOMwebClient {
    type QueryBuilder: DICOMQueryBuilder;

    fn default_headers(self, key: &'static str, value: &str) -> Self;

    fn search_studies(&mut self) -> Self::QueryBuilder {
        let url = format!("{}/studies", self.get_qido_prefix());
        info!("get url {}", &url);
        self.get_url(&url).header("Accept", "application/json")
    }

    fn search_series(&mut self, study_instance_uid: &str) -> Self::QueryBuilder {
        let url = format!(
            "{}/studies/{}/series",
            self.get_qido_prefix(),
            study_instance_uid
        );
        info!("get url {}", &url);
        self.get_url(&url).header("Accept", "application/json")
    }

    fn search_instances(
        &mut self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> Self::QueryBuilder {
        let url = format!(
            "{}/studies/{}/series/{}/instances",
            self.get_qido_prefix(),
            study_instance_uid,
            series_instance_uid,
        );
        info!("get url {}", &url);
        self.get_url(&url).header("Accept", "application/json")
    }

    fn retrieve_study(&mut self, study_instance_uid: &str) -> Self::QueryBuilder {
        todo!()
    }

    fn retrieve_series(
        &mut self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> Self::QueryBuilder {
        todo!()
    }

    fn retrieve_instance(
        &mut self,
        study_instance_uid: &str,
        series_instance_uid: &str,
        sop_instance_uid: &str,
    ) -> Self::QueryBuilder {
        let url = format!(
            "{}/studies/{}/series/{}/instances/{}",
            self.get_wado_prefix(),
            study_instance_uid,
            series_instance_uid,
            sop_instance_uid,
        );
        info!("get url {}", &url);
        self.get_url(&url)
            .header("Accept", "multipart/related; type=\"application/dicom\"")
    }

    fn store_instances(&mut self) -> Self::QueryBuilder {
        todo!();
    }

    fn get_url(&mut self, url: &str) -> Self::QueryBuilder;
    fn get_qido_prefix(&self) -> &str;
    fn get_wado_prefix(&self) -> &str;
}

pub trait DICOMQueryBuilder {
    fn query(self, key: &str, value: &str) -> Self;
    fn header(self, key: &str, value: &str) -> Self;

    fn patient_name(self, name_query: &str) -> Self
    where
        Self: Sized,
    {
        self.query("PatientName", name_query)
    }

    fn limit(self, limit: u32) -> Self
    where
        Self: Sized,
    {
        self.query("limit", limit.to_string().as_str())
    }

    fn offset(self, offset: u32) -> Self
    where
        Self: Sized,
    {
        self.query("offset", offset.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
