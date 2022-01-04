use bytes::{Buf, Bytes};
use dicom::object::{DefaultDicomObject, InMemDicomObject, StandardDataDictionary};
use log::{debug, error, trace};
use serde_json::Value;
use std::io::{BufRead, Write};
use std::io::{Cursor, Read};
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

pub fn multipart_encode_binary(buffer: Vec<u8>, boundary: &str) -> Vec<u8> {
    let nbytes = buffer.len();
    let mut body_header = Cursor::new(Vec::with_capacity(4 * 80));
    write!(body_header, "--{}\r\n", boundary).unwrap();
    write!(
        body_header,
        "Content-Type: multipart/related; type=\"application/dicom\"; boundary={}\r\n",
        boundary
    )
    .unwrap();
    write!(body_header, "Content-Length: {}\r\n", nbytes).unwrap();
    write!(body_header, "\r\n").unwrap();

    let mut body_payload = Cursor::new(buffer);
    body_payload.set_position(nbytes as u64);

    write!(body_payload, "\r\n--{}--", boundary).unwrap();

    let mut body = body_header.into_inner();
    let payload_vec = body_payload.into_inner();
    body.extend(payload_vec);
    body
}

pub fn multipart_encode(mut dicoms: Vec<DefaultDicomObject>, boundary: &str) -> Vec<u8> {
    assert!(dicoms.len() == 1);
    let obj = dicoms.remove(0);
    let mut body_payload = Cursor::new(Vec::with_capacity(1024 * 1024));
    obj.write_all(&mut body_payload).unwrap();

    multipart_encode_binary(body_payload.into_inner(), boundary)
}

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
                    assert!(buffer.ends_with("--".as_bytes()));
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
    let result = DefaultDicomObject::from_reader(file)?;
    Ok(result)
}

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
