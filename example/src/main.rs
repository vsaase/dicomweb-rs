use client::DICOMWebClient;
use reqwest;
// use error_chain::error_chain;

// error_chain! {
//     foreign_links {
//         Io(std::io::Error);
//         HttpRequest(reqwest::Error);
//     }
// }

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    // let url = "http://localhost:8088/rs";
    let url = "http://hackathon.siim.org/dicomweb/studies";
    // let client = DICOMWebClient::new(url);
    // let query = QueryParameters::patient("Master*");
    // client.query_studies(query).await?;
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("apikey", "9c8a1e06-9b19-4e36-81ff-3ece53bdb674")
        .query(&[("PatientName", "*"), ("limit", "10")])
        .send()
        .await?;
    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    let body = res.text().await?;
    println!("Body:\n{}", body);
    Ok(())
}
