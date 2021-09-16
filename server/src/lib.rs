use std::collections::{BTreeMap, HashMap};

use async_std::io;
use async_std::path::Path;
use async_trait::async_trait;
use dicom::core::header::Header;
use dicom::core::Tag;
use dicom::object::open_file;
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use dicomweb_util::encode::encode_dicom_to_json;
use log::info;
use serde_json::{json, Value};
use tide::log::debug;

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
pub struct DICOMwebServer<T> {
    app: tide::Server<T>,
}

impl<T> DICOMwebServer<T>
where
    T: DICOMServer + Clone + Send + Sync + 'static,
{
    pub fn with_dicom_server(server: T) -> Self {
        let mut app = tide::with_state(server);
        app.at(&("/".to_string()
            + &app.state().get_qido_prefix()
            + if !app.state().get_qido_prefix().is_empty() {
                "/"
            } else {
                ""
            }
            + "studies"))
            .get(Self::find_studies);
        DICOMwebServer { app }
    }

    async fn find_studies(mut req: tide::Request<T>) -> tide::Result {
        let server = req.state();
        let dicoms = server.find_studies().await;

        let res: Vec<BTreeMap<String, HashMap<String, Value>>> =
            dicoms.into_iter().map(encode_dicom_to_json).collect();
        Ok(json!(res).into())
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

    async fn find_studies(&self) -> Vec<InMemDicomObject>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
