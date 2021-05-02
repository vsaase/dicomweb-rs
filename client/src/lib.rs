use dicom::core::Tag;
use reqwest;
use std::collections::HashMap;

#[derive(Default)]
pub struct DICOMWebClient {
    client: reqwest::Client,
    url: String,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl DICOMWebClient {
    pub fn new(url: &str) -> DICOMWebClient {
        DICOMWebClient {
            client: reqwest::Client::new(),
            url: String::from(url),
            ..Default::default()
        }
    }

    pub fn query_studies(&self) -> QueryBuilder {
        QueryBuilder {
            client: &self,
            request_builder: self.client.get(&self.url),
        }
    }

    // pub async fn query_studies(self, query: QueryParameters) -> Result<(), reqwest::Error> {
    //     let client = reqwest::Client::new();
    //     let mut req = client.get(self.url).query(&query.values);
    //     if let Some(limit) = query.limit {
    //         req = req.query(&[("limit", limit)]);
    //     }
    //     let res = req.send().await?;
    //     println!("Status: {}", res.status());
    //     println!("Headers:\n{:#?}", res.headers());

    //     let body = res.text().await?;
    //     println!("Body:\n{}", body);
    //     Ok(())
    // }
}

pub struct QueryBuilder<'a> {
    client: &'a DICOMWebClient,
    request_builder: reqwest::RequestBuilder,
}

pub struct QueryParameters {
    values: HashMap<String, String>,
    includefield: Option<Vec<Tag>>,
    fuzzymatching: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl QueryParameters {
    pub fn patient(name: &str) -> QueryParameters {
        let mut values = HashMap::new();
        values.insert(String::from("00100010"), String::from(name));
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
