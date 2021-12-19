use async_std::path::Path;
use async_trait::async_trait;
use dicom::object::{open_file, DefaultDicomObject, InMemDicomObject};
use dicomweb_server::{DICOMServer, DICOMwebServer, INSTANCETAGS, SERIESTAGS, STUDYTAGS};
use itertools::Itertools;
use walkdir::WalkDir;

#[derive(Clone, Default)]
struct Server {
    dicoms: Vec<DefaultDicomObject>,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl Server {
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
    fn get_wado_prefix(&self) -> &str {
        &self.wado_url_prefix
    }

    async fn search_studies(&self) -> Vec<InMemDicomObject> {
        self.dicoms
            .iter()
            .unique_by(|d| {
                d.element_by_name("StudyInstanceUID")
                    .unwrap()
                    .to_str()
                    .unwrap()
            })
            .map(|d| {
                InMemDicomObject::from_element_iter(
                    d.clone()
                        .into_inner()
                        .into_iter()
                        .filter(|elt| STUDYTAGS.contains(&elt.header().tag))
                        .map(|elt| elt.clone()),
                )
            })
            .collect()
    }

    async fn search_series(&self, study_instance_uid: &str) -> Vec<InMemDicomObject> {
        self.dicoms
            .iter()
            .filter(|d| {
                d.element_by_name("StudyInstanceUID")
                    .unwrap()
                    .to_clean_str()
                    .unwrap()
                    == study_instance_uid
            })
            .unique_by(|d| {
                d.element_by_name("SeriesInstanceUID")
                    .unwrap()
                    .to_str()
                    .unwrap()
            })
            .map(|d| {
                InMemDicomObject::from_element_iter(
                    d.clone()
                        .into_inner()
                        .into_iter()
                        .filter(|elt| {
                            STUDYTAGS.contains(&elt.header().tag)
                                || SERIESTAGS.contains(&elt.header().tag)
                        })
                        .map(|elt| elt.clone()),
                )
            })
            .collect()
    }

    async fn search_instances(
        &self,
        study_instance_uid: &str,
        series_instance_uid: &str,
    ) -> Vec<InMemDicomObject> {
        self.dicoms
            .iter()
            .filter(|d| {
                d.element_by_name("StudyInstanceUID")
                    .unwrap()
                    .to_clean_str()
                    .unwrap()
                    == study_instance_uid
            })
            .filter(|d| {
                d.element_by_name("SeriesInstanceUID")
                    .unwrap()
                    .to_clean_str()
                    .unwrap()
                    == series_instance_uid
            })
            .map(|d| {
                InMemDicomObject::from_element_iter(
                    d.clone()
                        .into_inner()
                        .into_iter()
                        .filter(|elt| {
                            STUDYTAGS.contains(&elt.header().tag)
                                || SERIESTAGS.contains(&elt.header().tag)
                                || INSTANCETAGS.contains(&elt.header().tag)
                        })
                        .map(|elt| elt.clone()),
                )
            })
            .collect()
    }

    async fn retrieve_instance(
        &self,
        _study_instance_uid: &str,
        _series_instance_uid: &str,
        sop_instance_uid: &str,
    ) -> Option<DefaultDicomObject> {
        self.dicoms
            .iter()
            .filter(|d| {
                d.element_by_name("SOPInstanceUID")
                    .unwrap()
                    .to_clean_str()
                    .unwrap()
                    == sop_instance_uid
            })
            .next()
            .map(|d| d.clone())
    }
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let server = Server::from_dir(Path::new(format!("{}/dicoms", env!("HOME")).as_str()));

    let address = "127.0.0.1:8081";
    println!("listening on {}", address);
    let web_server = DICOMwebServer::with_dicom_server(server);
    web_server.listen(address).await?;
    Ok(())
}
