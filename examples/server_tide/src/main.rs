use async_std::path::Path;
use dicomweb_server::{DICOMServer, DICOMwebServer};
use log::info;
use walkdir::WalkDir;
use async_trait::async_trait;
use dicom::object::{DefaultDicomObject, open_file};


#[derive(Clone, Default)]
struct Server {
    dicoms: Vec<DefaultDicomObject>,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl Server {
    pub fn new() -> Server {
        Server::with_dicoms(vec![])
    }

    pub fn with_dicoms(dicoms: Vec<DefaultDicomObject>) -> Server {
        Server {
            dicoms,
            ..Default::default()
        }
    }

    pub fn from_dir(dir_path: &Path) -> Server {
        println!("walking directory {}", dir_path.to_str().unwrap());
        let dicoms = WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|v| v.ok())
            .filter_map(|x| open_file(x.path()).ok())
            .collect();
        // .for_each(|x| println!("{}", x.path().display()));
        Server::with_dicoms(dicoms)
    }

}

#[async_trait]
impl DICOMServer for Server {
    type State = Server;


    fn get_qido_prefix(&self) -> &str {
        &self.qido_url_prefix
    }

    async fn find_studies(&self) -> Vec<DefaultDicomObject>{
        todo!()
    }

}


#[async_std::main]
async fn main() -> tide::Result<()> {
    let server = Server::from_dir(Path::new("/Users/vsaase/Desktop/Saase_Armin"));
    let address = "127.0.0.1:8080";
    println!("listening on {}", address);
    let web_server = DICOMwebServer::with_dicom_server(server);
    web_server.listen(address).await?;
    Ok(())
}