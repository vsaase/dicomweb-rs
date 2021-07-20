use bytes::{Buf, Bytes};
use dicom::object::mem::InMemElement;
use dicom::object::{
    DefaultDicomObject, FileMetaTable, FileMetaTableBuilder, InMemDicomObject,
    StandardDataDictionary,
};
use enum_as_inner::EnumAsInner;
use error_chain::error_chain;
use log::{debug, error, info, log_enabled, trace, warn, Level};
use serde::Deserialize;
use serde_json::Value;
use std::{borrow::Borrow, io::BufRead};
use std::{
    collections::HashMap,
    io::{self, Cursor, Read},
};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Serde(serde_json::Error);
        Dicom(dicom::object::Error);
        DicomMeta(dicom::object::meta::Error);
        DicomCastValue(dicom::core::value::CastValueError);
    }

    errors{
        Custom(t: String) {
            description("custom")
            display("{}", t)
        }
    }
}

pub fn parse_multipart_body(body: Bytes, boundary: &str) -> Result<Vec<Vec<u8>>> {
    let mut reader = Cursor::new(body).reader();
    let mut line = String::new();

    let mut result = vec![];

    let mut state = "finding begin";
    let mut content_length: usize = 0;
    loop {
        match reader.read_line(&mut line) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                trace!("{}", state);
                trace!("{}", line);
                match state {
                    "finding begin" => {
                        if line.trim().ends_with(boundary) {
                            debug!("found start of part in multipart body");
                            state = "in header, need content length";
                        }
                    }
                    "in header, need content length" => {
                        if line.starts_with("Content-Length") {
                            content_length =
                                usize::from_str_radix(line.split_whitespace().last().unwrap(), 10)
                                    .unwrap();
                            debug!("content length:{}", content_length);
                            state = "in header, wait for end";
                        } else if line.trim() == "" {
                            state = "binary data starts";
                        }
                    }
                    "in header, wait for end" => {
                        if line.trim() == "" {
                            state = "binary data starts";
                        }
                    }
                    _ => {}
                }

                line.clear();
            }
            Err(err) => {
                error!("{}", err);
                break;
            }
        };
        match state {
            "binary data starts" => {
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
                state = "finding begin"
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
        Err(ErrorKind::Custom(String::from("error reading dicom")).into())
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

pub fn json2dicom(injson: &str) -> Result<InMemDicomObject<StandardDataDictionary>> {
    let parsed: DICOMJson = serde_json::from_str(injson)?;
    println!("{:?}", parsed);

    let ds = InMemDicomObject::create_empty();
    Ok(ds)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
