use dicom::object::DefaultDicomObject;
use dicomweb_util::multipart_encode_binary;
use log::info;
use thiserror::Error;

#[cfg(feature = "surf")]
pub mod async_surf;

pub mod reqwest;

/// The Error type of this crate with automatic translations from dependencies using the thiserror crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "surf")]
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

/// The central trait of the DICOMweb client library, which is implemented by the HTTP backend libraries.
/// The associated type `QueryBuilder` shall be set to a type that implements the DICOMQueryBuilder trait.
pub trait DICOMwebClient {
    type QueryBuilder: DICOMQueryBuilder;

    fn default_headers(self, key: &'static str, value: &str) -> Self;

    fn search_studies(&mut self) -> Self::QueryBuilder {
        let url = format!("{}/studies", self.get_qido_prefix());
        info!("get url {}", &url);
        self.get_url(&url)
            .header("Accept", "application/dicom+json")
    }

    fn search_series(&mut self, study_instance_uid: &str) -> Self::QueryBuilder {
        let url = format!(
            "{}/studies/{}/series",
            self.get_qido_prefix(),
            study_instance_uid
        );
        info!("get url {}", &url);
        self.get_url(&url)
            .header("Accept", "application/dicom+json")
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
        self.get_url(&url)
            .header("Accept", "application/dicom+json")
    }

    fn retrieve_study(&mut self, study_instance_uid: &str) -> Self::QueryBuilder {
        let url = format!("{}/studies/{}", self.get_wado_prefix(), study_instance_uid,);
        info!("get url {}", &url);
        self.get_url(&url)
            .header("Accept", "multipart/related; type=\"application/dicom\"")
    }

    fn retrieve_series(
        &mut self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> Self::QueryBuilder {
        let url = format!(
            "{}/studies/{}/series/{}",
            self.get_wado_prefix(),
            study_instance_uid,
            series_instance_uid,
        );
        info!("get url {}", &url);
        self.get_url(&url)
            .header("Accept", "multipart/related; type=\"application/dicom\"")
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
        let url = format!("{}/studies", self.get_stow_prefix());
        info!("post url {}", &url);
        let boundary = "ab69a3d5-542c-49e1-884b-8e135e104893";
        self.set_boundary(&boundary);
        let content_type = format!(
            "multipart/related; type=\"application/dicom\"; boundary={}",
            boundary
        );
        self.post_url(&url).header("content-type", &content_type)
    }

    fn get_url(&mut self, url: &str) -> Self::QueryBuilder;
    fn post_url(&mut self, url: &str) -> Self::QueryBuilder;
    fn set_boundary(&mut self, boundary: &str);
    fn get_boundary(&self) -> String;
    fn get_qido_prefix(&self) -> &str;
    fn get_wado_prefix(&self) -> &str;
    fn get_stow_prefix(&self) -> &str;
}

/// Every backend needs to implement this trait for a type that keeps track of
/// the building of a single query. This then serves as the associated type
/// in the `DICOMwebClient` trait.
///
/// The following methods are not part of this trait since their signature
/// depends on whether the backend library is async or blocking.
/// They have to be implemented in the types own impl block.
///
/// pub async fn results(self) -> Result<Vec<InMemDicomObject>>
/// pub async fn dicoms(self) -> Result<Vec<DefaultDicomObject>>
///
/// or
///
/// pub fn results(self) -> Result<Vec<InMemDicomObject>>
/// pub fn dicoms(self) -> Result<Vec<DefaultDicomObject>>
pub trait DICOMQueryBuilder {
    fn query(self, key: &str, value: &str) -> Self;
    fn header(self, key: &str, value: &str) -> Self;
    fn body(self, body: Vec<u8>) -> Self;
    fn get_boundary(&self) -> String;

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

    fn add_instance_buffer(self, buffer: Vec<u8>) -> Self
    where
        Self: Sized,
    {
        let body = multipart_encode_binary(buffer, &self.get_boundary());
        self.body(body)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
