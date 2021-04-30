use client::DICOMWebClient;
// use error_chain::error_chain;

// error_chain! {
//     foreign_links {
//         Io(std::io::Error);
//         HttpRequest(reqwest::Error);
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    let url = "http://localhost:8088/rs";
    let client = DICOMWebClient::new(url);
    let query = QueryParameters::patient("Master*");
    client.query_studies(query).await?;
    Ok(())
}
