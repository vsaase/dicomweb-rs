use http::{self, HeaderMap};

use serde::Serialize;

use error_chain::error_chain;

pub use dicomweb_util::DICOMJson;

pub mod async_surf;
pub mod reqwest;

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

pub trait DICOMWebClient {
    type QueryBuilder;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
