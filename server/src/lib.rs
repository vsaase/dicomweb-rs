use async_std::io;
use async_std::path::Path;
use dicom::object::open_file;
use dicom::object::DefaultDicomObject;
use log::info;
use tide::prelude::*;
use tide::Request;
use walkdir::WalkDir;

pub struct DICOMWebServer<'a> {
    app: tide::Server<&'a DICOMWebServer<'a>>,
    dicoms: Vec<DefaultDicomObject>,
    qido_url_prefix: String,
    wado_url_prefix: String,
    stow_url_prefix: String,
    ups_url_prefix: String,
}

impl<'a> DICOMWebServer<'a> {
    pub fn new() -> DICOMWebServer<'a> {
        println!("making new DICOMWebServer");
        let qido_url_prefix = "".to_string();
        let mut app = tide::new();
        app.at(&("/".to_string()
            + &qido_url_prefix
            + if !qido_url_prefix.is_empty() { "/" } else { "" }
            + "studies"))
            .get(Self::find_studies);

        DICOMWebServer {
            app,
            dicoms: vec![],
            qido_url_prefix,
            wado_url_prefix: String::default(),
            stow_url_prefix: String::default(),
            ups_url_prefix: String::default(),
        }
    }

    pub fn from_dir(dir_path: &Path) -> DICOMWebServer<'a> {
        let mut server = DICOMWebServer::new();
        println!("walking directory {}", dir_path.to_str().unwrap());
        let dicoms = WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|v| v.ok())
            .filter_map(|x| open_file(x.path()).ok())
            .collect();
        // .for_each(|x| println!("{}", x.path().display()));
        server.dicoms = dicoms;
        server
    }

    pub async fn listen(self, listener: &str) -> io::Result<()> {
        self.app.listen(listener).await?;
        Ok(())
    }

    async fn find_studies(mut req: Request<()>) -> tide::Result<serde_json::Value> {
        let obj = self.dicoms[0];
        Ok(json!([{
            "00080005" :
            {
                "Value" :
                [
                    "ISO_IR 100"
                ],
                "vr" : "CS"
            },
            "00080020" :
            {
                "Value" :
                [
                    "20210414"
                ],
                "vr" : "DA"
            },
            "00080030" :
            {
                "Value" :
                [
                    "074712.513000"
                ],
                "vr" : "TM"
            },
            "00080050" :
            {
                "Value" :
                [
                    "115081"
                ],
                "vr" : "SH"
            },
            "00080061" :
            {
                "Value" :
                [
                    "MR",
                    "SR"
                ],
                "vr" : "CS"
            },
            "00080090" :
            {
                "Value" :
                [
                    {
                        "Alphabetic" : "A. G\u{00f6}ttelmann"
                    }
                ],
                "vr" : "PN"
            },
            "00081190" :
            {
                "Value" :
                [
                    "http://localhost:8042/dicom-web/studies/1.2.276.0.110.1.210365.20210414074227000.115081"
                ],
                "vr" : "UR"
            },
            "00100010" :
            {
                "Value" :
                [
                    {
                        "Alphabetic" : "Saase^Armin"
                    }
                ],
                "vr" : "PN"
            },
            "00100020" :
            {
                "Value" :
                [
                    "16806"
                ],
                "vr" : "LO"
            },
            "00100030" :
            {
                "Value" :
                [
                    "19520502"
                ],
                "vr" : "DA"
            },
            "00100040" :
            {
                "Value" :
                [
                    "M"
                ],
                "vr" : "CS"
            },
            "0020000D" :
            {
                "Value" :
                [
                    "1.2.276.0.110.1.210365.20210414074227000.115081"
                ],
                "vr" : "UI"
            },
            "00200010" :
            {
                "Value" :
                [
                    "0"
                ],
                "vr" : "SH"
            },
            "00201206" :
            {
                "Value" :
                [
                    6
                ],
                "vr" : "IS"
            },
            "00201208" :
            {
                "Value" :
                [
                    72
                ],
                "vr" : "IS"
            }
        }
        ]))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}