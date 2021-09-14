use async_std::io;
use async_std::path::Path;
use dicom::object::open_file;
use dicom::object::DefaultDicomObject;
use log::info;
use async_trait::async_trait;


pub struct DICOMwebServer<T> {
    app: tide::Server<T>,
}

impl<T> DICOMwebServer<T>
where T: DICOMServer + Clone + Send + Sync + 'static{
    pub fn with_dicom_server(server: T) -> Self{
        let app = tide::with_state(server);
        DICOMwebServer{app}
    }

    fn init_app(&mut self) {
        let server = self.app.state();
        self.app.at(&("/".to_string()
            + &server.get_qido_prefix()
            + if !server.get_qido_prefix().is_empty() { "/" } else { "" }
            + "studies")).get(Self::find_studies);
    }

    async fn find_studies(mut req: tide::Request<T>) -> tide::Result {
        let server = req.state();
        let dicoms = server.find_studies();
        Ok("".into())
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

    async fn find_studies(&self) -> Vec<DefaultDicomObject>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
