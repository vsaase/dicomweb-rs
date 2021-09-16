use bytes::{Buf, Bytes};
use dicom::object::mem::InMemElement;
use dicom::object::{
    DefaultDicomObject, FileMetaTable, FileMetaTableBuilder, InMemDicomObject,
    StandardDataDictionary,
};
use enum_as_inner::EnumAsInner;
use log::{debug, error, info, log_enabled, trace, warn, Level};
use serde::Deserialize;
use serde_json::Value;
use std::{borrow::Borrow, io::BufRead};
use std::{
    collections::HashMap,
    io::{self, Cursor, Read},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Dicom(#[from] dicom::object::Error),
    #[error("{0}")]
    DicomCastValue(#[from] dicom::core::value::CastValueError),
    #[error("{0}")]
    Custom(String),
}
pub type Result<T> = std::result::Result<T, Error>;

pub mod decode;
pub mod encode;

#[derive(Debug)]
enum MultipartParserStates {
    NextPart,
    InHeader,
    InBinary,
}

pub fn parse_multipart_body(body: Bytes, boundary: &str) -> Result<Vec<Vec<u8>>> {
    let mut reader = Cursor::new(body).reader();
    let mut line = String::new();

    let mut result = vec![];

    let mut state = MultipartParserStates::NextPart;
    let mut content_length: usize = 0;

    loop {
        match reader.read_line(&mut line) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                trace!("{:?}", state);
                trace!("{:?}", line);
                match state {
                    MultipartParserStates::NextPart => {
                        if line.trim().ends_with(boundary) {
                            debug!("found start of part in multipart body");
                            state = MultipartParserStates::InHeader;
                        }
                    }
                    MultipartParserStates::InHeader => {
                        if line.starts_with("Content-Length") {
                            content_length =
                                usize::from_str_radix(line.split_whitespace().last().unwrap(), 10)
                                    .unwrap();
                            debug!("content length:{}", content_length);
                        } else if line.trim() == "" {
                            state = MultipartParserStates::InBinary;
                        }
                    }
                    _ => {
                        return Err(Error::Custom(
                            "in wrong state when reading multipart header".to_string(),
                        ));
                    }
                }

                line.clear();
            }
            Err(err) => {
                error!("{}", err);
                break;
            }
        };
        match state {
            MultipartParserStates::InBinary => {
                if content_length > 0 {
                    let mut buffer = vec![0u8; content_length];
                    reader.read_exact(&mut buffer)?;
                    result.push(buffer);
                } else {
                    // length not specified, assuming single part and trailing boundary like CRLF--boundary--
                    let mut buffer = Vec::new();
                    reader.read_to_end(&mut buffer)?;
                    let len = buffer.len() - boundary.len() - 6;
                    result.push(buffer[..len].into());
                }
                state = MultipartParserStates::NextPart
            }
            _ => {}
        }
    }
    Ok(result)
}

pub fn dicom_from_reader<R: Read>(mut file: R) -> Result<DefaultDicomObject> {
    // skip preamble
    {
        let mut buf = [0u8; 128];
        // skip the preamble
        file.read_exact(&mut buf)?;
    }
    let result = DefaultDicomObject::from_reader(file);
    if let Ok(ds) = result {
        Ok(ds)
    } else {
        Err(Error::Custom(String::from("error reading dicom")).into())
    }
}

#[derive(Debug, Deserialize, EnumAsInner)]
#[serde(untagged)]
pub enum DICOMJsonTagValue {
    Str(String),
    Int(i32),
    DICOMJson(DICOMJson),
    Value(Value),
}

#[derive(Debug, Deserialize)]
pub struct DICOMJsonTag {
    pub vr: String,
    pub Value: Vec<DICOMJsonTagValue>,
}

pub type DICOMJson = Vec<HashMap<String, DICOMJsonTag>>;
pub type DicomResponse = InMemDicomObject<StandardDataDictionary>;

pub fn json2dicom(parsed: &Vec<Value>) -> Result<Vec<DicomResponse>> {
    // let parsed: Vec<Value> = serde_json::from_str(injson)?;
    println!("{:?}", parsed);

    let ds = parsed.iter().map(decode::decode_response_item).collect();
    Ok(ds)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
