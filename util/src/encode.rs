use std::collections::{BTreeMap, HashMap};

use dicom::core::VR::*;
use dicom::object::InMemDicomObject;
use serde_json::{json, Value};

pub type DICOMJsonObject = BTreeMap<String, HashMap<String, Value>>;

// http://dicom.nema.org/medical/dicom/current/output/chtml/part18/sect_F.2.3.html#table_F.2.3-1
pub fn encode_dicom_to_json(dicom: InMemDicomObject) -> DICOMJsonObject {
    dicom
        .into_iter()
        .map(|elt| {
            let mut eltmap = HashMap::new();
            eltmap.insert("vr".to_string(), json!(elt.header().vr().to_string()));
            eltmap.insert("Value".to_string(), {
                match elt.header().vr() {
                    AE | AS | AT | CS | DA | DS | DT | IS | LO | LT | SH | ST | SV | TM | UC
                    | UI | UR | UT | UV => match elt.value().multiplicity() {
                        0 => json!([]),
                        1 => json!([elt.value().to_clean_str().unwrap()]),
                        _ => json!(elt.value().to_multi_str().unwrap()),
                    },
                    FL => {
                        json!([elt.value().to_float32().unwrap()])
                    }
                    FD => {
                        json!([elt.value().to_float64().unwrap()])
                    }
                    OB | OD | OF | OL | OV | OW | UN => {
                        let bytes = elt.value().to_bytes().unwrap();
                        json!(base64::encode(bytes))
                    }
                    PN => {
                        json!([{ "Alphabetic": elt.value().to_clean_str().unwrap() }])
                    }
                    SL => {
                        json!([elt.value().to_int::<i64>().unwrap()])
                    }
                    SQ => match elt.value() {
                        dicom::core::DicomValue::Sequence { items, size: _ } => {
                            let v: Vec<DICOMJsonObject> = items
                                .into_iter()
                                .map(|i| encode_dicom_to_json(i.clone()))
                                .collect();
                            json!(v)
                        }
                        _ => panic!(),
                    },
                    SS => {
                        json!([elt.value().to_int::<i32>().unwrap()])
                    }
                    UL => {
                        json!([elt.value().to_int::<u64>().unwrap()])
                    }
                    US => {
                        json!([elt.value().to_int::<u32>().unwrap()])
                    }
                }
            });
            (
                format!(
                    "{:04X}{:04X}",
                    elt.header().tag.group(),
                    elt.header().tag.element()
                ),
                eltmap,
            )
        })
        .collect()
}
