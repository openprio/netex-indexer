extern crate walkdir;

use crate::xml_util::read_to_end_into_buffer;

use time::format_description::well_known::Iso8601;
use time::PrimitiveDateTime;

use std::str;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use time::Date;

use quick_xml::de::Deserializer;

use ::serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct XMLVersion {
    start_date: String,
    end_date: String,
    version_type: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct XMLValidBetween {
    from_date: String,
    to_date: String,
}

impl XMLVersion {
    fn convert_to_version(&self) -> ValidBetween {
        let start_date: PrimitiveDateTime =
            PrimitiveDateTime::parse(&self.start_date, &Iso8601::DEFAULT).unwrap();
        let end_date: PrimitiveDateTime =
            PrimitiveDateTime::parse(&self.end_date, &Iso8601::DEFAULT).unwrap();

        return ValidBetween {
            start_date: start_date.date(),
            end_date: end_date.date(),
            version_type: self.version_type.clone(),
        };
    }
}

impl XMLValidBetween {
    fn convert_to_valid_between(&self) -> ValidBetween {
        let start_date: PrimitiveDateTime =
            PrimitiveDateTime::parse(&self.from_date, &Iso8601::DEFAULT).unwrap();
        let end_date: PrimitiveDateTime =
            PrimitiveDateTime::parse(&self.to_date, &Iso8601::DEFAULT).unwrap();

        println!("Parsed ValidBetween from {} to {}", start_date, end_date);

        return ValidBetween {
            start_date: start_date.date(),
            end_date: end_date.date(),
            version_type: "baseline".to_string(),
        };
    }
}

#[derive(Debug)]
pub struct ValidBetween {
    start_date: Date,
    end_date: Date,
    version_type: String,
}

#[derive(Debug, Clone)]
pub struct NeTExVersion {
    pub file_name: String,
    pub data_owner_code: String,
    pub publication_time_stamp: PrimitiveDateTime,
    pub partition: String,
    pub netex_version: String,
    pub start_date: Date,
    pub end_date: Date,
    pub version_type: String,
    pub path: String,
}

pub fn parse_netex(s: String, file_name: String, path: String) -> Option<NeTExVersion> {
    let mut reader = Reader::from_str(&s);
    let mut junk_buf: Vec<u8> = Vec::new();
    let mut buf = Vec::new();

    let mut valid_between = None;
    let mut netex_version = None;
    let mut partition = None;
    let mut publication_time_stamp = None;
    let mut data_owner_code = None;
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,
            Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"DefaultCodespaceRef" => {
                    let codespace_ref = e.attributes().find(|item| {
                        if let Ok(item) = item {
                            return item.key.0 == b"ref";
                        }
                        false
                    });
                    let reft = codespace_ref.unwrap().unwrap().unescape_value().unwrap();
                    data_owner_code = Some(reft.split(":").last().unwrap().to_string());
                }

                b"DefaultResponsibilitySetRef" => {
                    let responsibility_ref = e.attributes().find(|item| {
                        if let Ok(item) = item {
                            return item.key.0 == b"ref";
                        }
                        false
                    });
                    let reft = responsibility_ref
                        .unwrap()
                        .unwrap()
                        .unescape_value()
                        .unwrap();
                    partition = Some(reft.to_string());
                }
                b"TypeOfFrameRef" => {
                    let netex_version_value = e.attributes().find(|item| {
                        if let Ok(item) = item {
                            return item.key.as_ref() == b"version";
                        }
                        false
                    });
                    println!("{}", file_name);
                    let version = netex_version_value
                        .unwrap()
                        .unwrap()
                        .unescape_value()
                        .unwrap();
                    netex_version = Some(version.to_string());
                }
                _ => {}
            },
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Version" => {
                    let result = read_to_end_into_buffer(&mut reader, &e, &mut junk_buf).unwrap();
                    let version_element = str::from_utf8(&result).unwrap();
                    let mut deserializer = Deserializer::from_str(version_element);
                    let xml_version = XMLVersion::deserialize(&mut deserializer).unwrap();
                    valid_between = Some(xml_version.convert_to_version());
                }
                b"ValidBetween" => {
                    let result = read_to_end_into_buffer(&mut reader, &e, &mut junk_buf).unwrap();
                    let version_element = str::from_utf8(&result).unwrap();
                    let mut deserializer = Deserializer::from_str(version_element);
                    let xml_valid_between =
                        XMLValidBetween::deserialize(&mut deserializer).unwrap();
                    valid_between = Some(xml_valid_between.convert_to_valid_between());
                }
                b"PublicationTimestamp" => match reader.read_event_into(&mut buf) {
                    Ok(Event::Text(e)) => {
                        publication_time_stamp = Some(
                            PrimitiveDateTime::parse(&e.unescape().unwrap(), &Iso8601::DEFAULT)
                                .unwrap(),
                        );
                    }
                    _ => {}
                },
                _ => (),
            },
            _ => (),
        }

        if valid_between.is_some()
            && netex_version.is_some()
            && partition.is_some()
            && publication_time_stamp.is_some()
            && data_owner_code.is_some()
        {
            let valid_between = valid_between.unwrap();
            return Some(NeTExVersion {
                file_name: file_name,
                data_owner_code: data_owner_code.unwrap(),
                publication_time_stamp: publication_time_stamp.unwrap(),
                partition: partition.unwrap(),
                netex_version: netex_version.unwrap(),
                start_date: valid_between.start_date,
                end_date: valid_between.end_date,
                version_type: valid_between.version_type,
                path: path,
            });
        }
        buf.clear();
    }
    return None;
}
