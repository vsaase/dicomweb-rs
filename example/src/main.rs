use client::DICOMWebClient;
use error_chain::error_chain;
use log::{debug, error, info, log_enabled, warn, Level};
use serde_json::Value;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(client::Error);
        Serde(serde_json::Error);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // let url = "http://localhost:8088/rs";
    let url = "http://hackathon.siim.org/dicomweb";
    let client = DICOMWebClient::new(url);
    info!("querying studies");
    let res = client
        .find_studies()
        .header("apikey", "9c8a1e06-9b19-4e36-81ff-3ece53bdb674")
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
        .header("apikey", "9c8a1e06-9b19-4e36-81ff-3ece53bdb674")
        .limit(10)
        .send()
        .await?;
    let json: Value = res.json().await?;
    println!("JSON body:\n{}", json);

    let series_instance_uid = json[0]["0020000E"]["Value"][0].as_str().unwrap();
    info!("querying instances");
    let res = client
        .find_instances(study_instance_uid, series_instance_uid)
        .header("apikey", "9c8a1e06-9b19-4e36-81ff-3ece53bdb674")
        .limit(10)
        .send()
        .await?;
    let json: Value = res.json().await?;
    println!("JSON body:\n{}", json);

    let sop_instance_uid = json[0]["00080018"]["Value"][0].as_str().unwrap();

    Ok(())
}
