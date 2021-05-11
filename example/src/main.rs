use std::io::Cursor;

use bytes::Buf;
use client::DICOMWebClient;
use error_chain::error_chain;
use log::{debug, error, info, log_enabled, trace, warn, Level};
use serde_json::Value;
use util::{dicom_from_reader, parse_multipart_body};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(client::Error);
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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // let url = "http://localhost:8088/rs";
    let url = "http://hackathon.siim.org/dicomweb";
    // let client = DICOMWebClient::new(url);
    let client = DICOMWebClient::builder(url)
        .default_headers("apikey", "9c8a1e06-9b19-4e36-81ff-3ece53bdb674")
        .build()
        .unwrap();
    info!("querying studies");
    let res = client
        .find_studies()
        .patient_name("*")
        .limit(10)
        .send()
        .await?;
    // println!("Status: {}", res.status());
    // println!("Headers:\n{:#?}", res.headers());
    let json: Value = res.json().await?;
    println!("JSON body:\n{}", json);

    let study_instance_uid = json[0]["0020000D"]["Value"][0].as_str().unwrap();

    info!("querying series");
    let res = client
        .find_series(study_instance_uid)
        .limit(10)
        .send()
        .await?;
    let json: Value = res.json().await?;
    println!("JSON body:\n{}", json);

    let series_instance_uid = json[0]["0020000E"]["Value"][0].as_str().unwrap();

    info!("querying instances");
    let res = client
        .find_instances(study_instance_uid, series_instance_uid)
        .limit(10)
        .send()
        .await?;
    // let body = res.text().await?;
    // println!("Text body:\n{}", body);
    let json: Value = res.json().await?;
    println!("JSON body:\n{}", json);

    let sop_instance_uid = json[0]["00080018"]["Value"][0].as_str().unwrap();
    let res = client
        .get_instance(study_instance_uid, series_instance_uid, sop_instance_uid)
        .send()
        .await?;

    println!("Status: {}", res.status());
    // println!("Headers:\n{:#?}", res.headers());
    let content_type = res.headers()["content-type"].to_str().unwrap();
    println!("content-type: {}", content_type);
    let (_, boundary) = content_type.rsplit_once("boundary=").unwrap();
    let boundary = String::from(boundary);
    println!("boundary: {}", boundary);

    let body = res.bytes().await?;
    let parts = parse_multipart_body(body, &boundary)?;
    let reader = Cursor::new(parts[0].clone()).reader();
    let ds = dicom_from_reader(reader)?;
    println!("{:?}", ds.element_by_name("PatientName")?.to_str()?);
    Ok(())
}
