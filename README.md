# dicomweb-rs

[![dicomweb-rs on crates.io](https://img.shields.io/crates/v/dicomweb.svg)](https://crates.io/crates/dicomweb)
[![dependency status](https://deps.rs/repo/github/vsaase/dicomweb-rs/status.svg)](https://deps.rs/repo/github/vsaase/dicomweb-rs)

WIP implementation of DICOMweb client (reqwest or surf backend) and server (tide backend).

An example of use follows. See also the provided [`examples`](examples).

```rust
use dicomweb_client::async_surf::Client;
use dicomweb_client::{DICOMQueryBuilder, DICOMwebClient, Result};

#[async_std::main]
async fn main() -> Result<()> {
    let url = "http://localhost:8080/dcm4chee-arc/aets/DCM4CHEE/rs";
    let mut client = Client::new(url);

    // results is a Vec of DICOM objects
    let results = client
        .search_studies()
        .patient_name("*")
        .limit(10)
        .results()
        .await?;
        
    let study_instance_uid = results[0].element_by_name("StudyInstanceUID")?.to_str()?;
}
```
