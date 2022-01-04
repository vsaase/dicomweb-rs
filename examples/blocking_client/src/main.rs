use dicomweb_client::Result;
use dicomweb_client::{reqwest::blocking_reqwest::Client, DICOMQueryBuilder, DICOMwebClient};
use log::info;

fn main() -> Result<()> {
    env_logger::init();

    // let url = "http://localhost:8088/rs";
    // let url = "http://localhost:8042/dicom-web";
    let url = "http://localhost:8080/dcm4chee-arc/aets/DCM4CHEE/rs";
    // let url = "http://localhost:8081";

    info!("creating client");
    let mut client = Client::new(url);
    info!("querying studies");

    let results = client
        .search_studies()
        .patient_name("*")
        .limit(10)
        .results()?;

    let study_instance_uid = results[0].element_by_name("StudyInstanceUID")?.to_str()?;
    println!("{}", study_instance_uid);

    info!("querying series");
    let results = client
        .search_series(&study_instance_uid)
        .limit(10)
        .results()?;

    let series_instance_uid = results[0].element_by_name("SeriesInstanceUID")?.to_str()?;

    info!("querying instances");
    let results = client
        .search_instances(&study_instance_uid, &series_instance_uid)
        .limit(10)
        .results()?;

    let sop_instance_uid = results[0].element_by_name("SOPInstanceUID")?.to_str()?;

    info!("getting instance");
    let dicoms = client
        .retrieve_instance(&study_instance_uid, &series_instance_uid, &sop_instance_uid)
        .dicoms()?;
    println!("{:?}", dicoms[0].element_by_name("PatientName")?.to_str()?);
    Ok(())
}
