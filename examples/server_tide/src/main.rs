use async_std::path::Path;
use dicomweb_server::DICOMWebServer;
use log::info;
use tide::prelude::*;
use tide::Request;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let server = DICOMWebServer::from_dir(Path::new("/Users/vsaase/Desktop/Saase_Armin"));
    let address = "127.0.0.1:8080";
    println!("listening on {}", address);
    server.listen(address).await?;
    Ok(())
}
