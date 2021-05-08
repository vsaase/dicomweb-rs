use client::DICOMWebClient;
use error_chain::error_chain;
use reqwest;
use serde_json::Value;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        Serde(serde_json::Error);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // let url = "http://localhost:8088/rs";
    let url = "http://hackathon.siim.org/dicomweb/studies";
    let client = DICOMWebClient::new(url);
    let res = client
        .find_studies()
        .header("apikey", "9c8a1e06-9b19-4e36-81ff-3ece53bdb674")
        .patient_name("*")
        .limit(10)
        .send()
        .await?;
    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    let json: Value = res.json().await?;
    println!("JSON body:\n{}", json);
    Ok(())
}
