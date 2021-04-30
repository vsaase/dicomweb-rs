use dicom::core::Tag;
use reqwest;
use std::collections::HashMap;

#[derive(Default)]
pub struct DICOMWebClient {
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMWebClient {
    pub fn new(url: &str) -> DICOMWebClient {
        DICOMWebClient {
            url: String::from(url),
            ..Default::default()
        }
    }

    pub async fn query_studies(self, query: QueryParameters) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        let querytuples = query.values.iter().collect();
        let res = client.get(self.url).query(querytuples).send().await?;
        println!("Status: {}", res.status());
        println!("Headers:\n{:#?}", res.headers());

        let body = res.text().await?;
        println!("Body:\n{}", body);
        Ok(())
    }
}

pub struct QueryParameters {
    values: HashMap<Tag, String>,
    includefield: Option<Vec<Tag>>,
    fuzzymatching: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl QueryParameters {
    pub fn patient(name: &str) -> QueryParameters {
        let mut values = HashMap::new();
        values.insert(Tag(0x0010, 0x0010), String::from(name));
        QueryParameters {
            values,
            includefield: None,
            fuzzymatching: None,
            limit: None,
            offset: None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
