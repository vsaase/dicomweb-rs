use std::io::Cursor;

use bytes::Buf;
use dicomweb_client::{blocking_reqwest::DICOMWebClientBlocking, DICOMWebClient};
use dicomweb_util::{dicom_from_reader, parse_multipart_body, DICOMJson, DICOMJsonTagValue};
use error_chain::error_chain;
use log::{debug, error, info, log_enabled, trace, warn, Level};
use serde_json::Value;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        Serde(serde_json::Error);
        Dicom(dicom::object::Error);
        DicomCastValue(dicom::core::value::CastValueError);
    }

    errors{
        Custom(t: String) {
            description("custom")
            display("{}", t)
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    // let url = "http://localhost:8088/rs";
    // let url = "http://hackathon.siim.org/dicomweb";
    let url = "http://localhost:8042/dicom-web";
    // let url = "http://localhost:8080/dcm4chee-arc/aets/DCM4CHEE/rs";
    // let client = DICOMWebClient::new(url);
    info!("creating client");
    let client = DICOMWebClientBlocking::builder(url)
        .default_headers("apikey", "9c8a1e06-9b19-4e36-81ff-3ece53bdb674")
        .build()
        .unwrap();
    info!("querying studies");
    let json: DICOMJson = client.find_studies().patient_name("*").limit(10).json()?;
    println!("JSON body:\n{:?}", json);

    // if let DICOMJsonTagValue::String(study_instance_uid) = &json[0]["0020000D"].Value[0] {
    //     println!("{}", study_instance_uid);
    // }
    let study_instance_uid = json[0]["0020000D"].Value[0].as_str().unwrap();
    println!("{}", study_instance_uid);

    info!("querying series");
    let json: DICOMJson = client.find_series(study_instance_uid).limit(10).json()?;
    println!("JSON body:\n{:?}", json);

    let series_instance_uid = json[0]["0020000E"].Value[0].as_str().unwrap();

    info!("querying instances");
    let json: DICOMJson = client
        .find_instances(study_instance_uid, series_instance_uid)
        .limit(10)
        .json()?;
    println!("JSON body:\n{:?}", json);

    let sop_instance_uid = json[0]["00080018"].Value[0].as_str().unwrap();

    info!("getting instance");
    let dicoms = client
        .get_instance(study_instance_uid, series_instance_uid, sop_instance_uid)
        .dicoms()
        .unwrap();
    println!("{:?}", dicoms[0].element_by_name("PatientName")?.to_str()?);
    Ok(())
}