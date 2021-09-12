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
}
pub type Result<T> = std::result::Result<T, Error>;

pub trait DICOMWebClient {
    type QueryBuilder: DICOMQueryBuilder;

    fn default_headers(self, key: &'static str, value: &str) -> Self;

    fn find_studies(&mut self) -> Self::QueryBuilder {
        let url = format!("{}/studies", self.get_qido_prefix());
        self.get_url(&url)
    }

    fn find_series(&mut self, study_instance_uid: &str) -> Self::QueryBuilder {
        let url = format!(
            "{}/studies/{}/series",
            self.get_qido_prefix(),
            study_instance_uid
        );
        self.get_url(&url)
    }

    fn find_instances(
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
        self.get_url(&url)
    }

    fn get_instance(
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
        self.get_url(&url)
    }

    fn get_url(&mut self, url: &str) -> Self::QueryBuilder;
    fn get_qido_prefix(&self) -> &str;
    fn get_wado_prefix(&self) -> &str;
}

pub trait DICOMQueryBuilder {
    fn patient_name(self, name_query: &str) -> Self;
    fn limit(self, limit: u32) -> Self;
    fn offset(self, offset: u32) -> Self;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
