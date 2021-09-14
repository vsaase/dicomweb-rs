use async_std::io;
use async_std::path::Path;
use dicom::object::open_file;
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use log::info;
use async_trait::async_trait;
use serde_json::json;


pub struct DICOMwebServer<T> {
    app: tide::Server<T>,
}

impl<T> DICOMwebServer<T>
where T: DICOMServer + Clone + Send + Sync + 'static{
    pub fn with_dicom_server(server: T) -> Self{
        let mut app = tide::with_state(server);
        app.at(&("/".to_string()
            + &app.state().get_qido_prefix()
            + if !app.state().get_qido_prefix().is_empty() { "/" } else { "" }
            + "studies")).get(Self::find_studies);
        DICOMwebServer{app}
    }

    async fn find_studies(mut req: tide::Request<T>) -> tide::Result {
        let server = req.state();
        let dicoms = server.find_studies().await;
        Ok(json!([{"0020000D":{"vr":"UI","Value":[dicoms[0].element_by_name("StudyInstanceUID")?.to_str()?.trim_matches(|c: char| c == '\x00')]}}]).into())
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
