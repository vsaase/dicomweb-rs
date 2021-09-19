use dicomweb_client::async_surf::Client;

// use dicomweb_client::reqwest::async_reqwest::Client;
use dicomweb_client::{DICOMQueryBuilder, DICOMwebClient, Result};
use log::info;

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();

    // let url = "http://localhost:8088/rs";
    // let url = "http://localhost:8042/dicom-web";
    // let url = "http://localhost:8080/dcm4chee-arc/aets/DCM4CHEE/rs";
    let url = "http://localhost:8081";

    info!("creating client");

    let mut client = Client::new(url);
    info!("querying studies");

    let results = client
        .search_studies()
        .patient_name("*")
        .limit(10)
        .results()
        .await?;

    let study_instance_uid = results[0].element_by_name("StudyInstanceUID")?.to_str()?;
    println!("{}", study_instance_uid);

    info!("querying series");
    let results = client
        .search_series(&study_instance_uid)
        .limit(10)
        .results()
        .await?;

    let series_instance_uid = results[0].element_by_name("SeriesInstanceUID")?.to_str()?;

    info!("querying instances");
    let results = client
        .search_instances(&study_instance_uid, &series_instance_uid)
        .limit(10)
        .results()
        .await?;

    let sop_instance_uid = results[0].element_by_name("SOPInstanceUID")?.to_str()?;

    info!("getting instance");
    let dicoms = client
        .retrieve_instance(&study_instance_uid, &series_instance_uid, &sop_instance_uid)
        .dicoms()
        .await?;
    println!("{:?}", dicoms[0].element_by_name("PatientName")?.to_str()?);
    Ok(())
}
