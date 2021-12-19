use std::io::Write;

use async_trait::async_trait;
use dicom::core::Tag;
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use dicomweb_util::encode::{encode_dicom_to_json, DICOMJsonObject};
use dicomweb_util::multipart_encode;
use http_types::headers::HeaderValue;
use serde_json::json;
use std::io::{self, Cursor};
use tide::security::{CorsMiddleware, Origin};
use tide::Response;

// http://dicom.nema.org/medical/dicom/current/output/chtml/part18/sect_10.6.html#table_10.6.1-5
pub const STUDYTAGS: [Tag; 9] = [
    Tag(0x0008, 0x0020),
    Tag(0x0008, 0x0030),
    Tag(0x0008, 0x0050),
    Tag(0x0008, 0x0061),
    Tag(0x0008, 0x0090),
    Tag(0x0010, 0x0010),
    Tag(0x0010, 0x0020),
    Tag(0x0020, 0x000D),
    Tag(0x0020, 0x0010),
];

pub const SERIESTAGS: [Tag; 6] = [
    Tag(0x0008, 0x0060),
    Tag(0x0020, 0x000E),
    Tag(0x0020, 0x0011),
    Tag(0x0040, 0x0244),
    Tag(0x0040, 0x0245),
    Tag(0x0040, 0x0275),
    // Tag(0x0040, 0x0009),
    // Tag(0x0040, 0x1001),
];

pub const INSTANCETAGS: [Tag; 3] = [
    Tag(0x0008, 0x0016),
    Tag(0x0008, 0x0018),
    Tag(0x0020, 0x0013),
];

pub struct DICOMwebServer<T> {
    app: tide::Server<T>,
}

impl<T> DICOMwebServer<T>
where
    T: DICOMServer + Clone + Send + Sync + 'static,
{
    pub fn with_dicom_server(server: T) -> Self {
        let mut app = tide::with_state(server);
        let cors = CorsMiddleware::new()
            .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
            .allow_origin(Origin::from("*"))
            .allow_credentials(false);

        app.with(cors);

        app.with(tide::log::LogMiddleware::new());
        let qido = app.state().get_qido_prefix().to_string()
            + if !app.state().get_qido_prefix().is_empty() {
                "/"
            } else {
                ""
            };
        let wado = app.state().get_wado_prefix().to_string()
            + if !app.state().get_wado_prefix().is_empty() {
                "/"
            } else {
                ""
            };

        app.at(&("/".to_string() + &qido + "studies"))
            .get(Self::search_studies);

        app.at(&("/".to_string() + &qido + "studies/:study_instance_uid/series"))
            .get(Self::search_series);

        app.at(&("/".to_string()
            + &qido
            + "studies/:study_instance_uid/series/:series_instance_uid/instances"))
            .get(Self::search_instances);

        app.at(&("/".to_string()
            + &wado
            + "studies/:study_instance_uid/series/:series_instance_uid/instances/:sop_instance_uid"))
            .get(Self::retrieve_instance);

        DICOMwebServer { app }
    }

    async fn search_studies(req: tide::Request<T>) -> tide::Result {
        let server = req.state();
        let dicoms = server.search_studies().await;

        let res: Vec<DICOMJsonObject> = dicoms.into_iter().map(encode_dicom_to_json).collect();

        let mut res = Response::from(json!(res));
        res.set_content_type("application/dicom+json");
        Ok(res)
    }

    async fn search_series(req: tide::Request<T>) -> tide::Result {
        let server = req.state();
        let study_instance_uid = req.param("study_instance_uid")?;
        let dicoms = server.search_series(study_instance_uid).await;

        let res: Vec<DICOMJsonObject> = dicoms.into_iter().map(encode_dicom_to_json).collect();

        let mut res = Response::from(json!(res));
        res.set_content_type("application/dicom+json");
        Ok(res)
    }

    async fn search_instances(req: tide::Request<T>) -> tide::Result {
        let server = req.state();
        let study_instance_uid = req.param("study_instance_uid")?;
        let series_instance_uid = req.param("series_instance_uid")?;
        let dicoms = server
            .search_instances(study_instance_uid, series_instance_uid)
            .await;

        let res: Vec<DICOMJsonObject> = dicoms.into_iter().map(encode_dicom_to_json).collect();

        let mut res = Response::from(json!(res));
        res.set_content_type("application/dicom+json");
        Ok(res)
    }

    async fn retrieve_instance(req: tide::Request<T>) -> tide::Result {
        let server = req.state();
        let study_instance_uid = req.param("study_instance_uid")?;
        let series_instance_uid = req.param("series_instance_uid")?;
        let sop_instance_uid = req.param("sop_instance_uid")?;
        let dicom = server
            .retrieve_instance(study_instance_uid, series_instance_uid, sop_instance_uid)
            .await;

        if let Some(obj) = dicom {
            let mut res = Response::new(200);
            let boundary = "ab69a3d5-542c-49e1-884b-8e135e104893";
            res.set_content_type(
                format!(
                    "multipart/related; type=\"application/dicom\"; boundary={}",
                    boundary
                )
                .as_str(),
            );

            let body = multipart_encode(vec![obj], boundary);

            res.set_body(body);
            Ok(res)
        } else {
            let res = Response::new(404);
            Ok(res)
        }
    }

    pub async fn listen(self, listener: &str) -> io::Result<()> {
        self.app.listen(listener).await?;
        Ok(())
    }
}

#[async_trait]
pub trait DICOMServer {
    type State: DICOMServer;

    fn get_qido_prefix(&self) -> &str;
    fn get_wado_prefix(&self) -> &str;

    async fn search_studies(&self) -> Vec<InMemDicomObject>;
    async fn search_series(&self, study_instance_uid: &str) -> Vec<InMemDicomObject>;
    async fn search_instances(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> Vec<InMemDicomObject>;

    async fn retrieve_instance(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
        sop_instance_uid: &str,
    ) -> Option<DefaultDicomObject>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
