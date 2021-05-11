use bytes::{Buf, Bytes};
use log::{debug, error, info, log_enabled, trace, warn, Level};
use std::io::{self, Cursor, Read};
use std::{borrow::Borrow, io::BufRead};

pub fn parse_multipart_body(body: Bytes, boundary: &str) {
    let mut reader = Cursor::new(body).reader();
    let mut line = String::new();

    let mut state = "finding begin";
    let mut content_length: usize = 0;
    loop {
        match reader.read_line(&mut line) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }

                println!("{}", line);
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
                let mut buffer = vec![0u8; content_length];
                reader.read_exact(&mut buffer).unwrap();
                state = "end of data"
            }
            _ => {}
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
