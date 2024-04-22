extern crate walkdir;

use crate::xml_util::read_to_end_into_buffer;
use time::macros::datetime;

use time::format_description::well_known::Iso8601;
use time::PrimitiveDateTime;

use std::{str, ops::Deref, io::Read};

use rayon::iter::ParallelIterator;

use quick_xml::events::{Event, BytesText};
use quick_xml::reader::Reader;
use time::Date;
//use serde::Deserialize;
use quick_xml::de::{Deserializer, XmlRead};
// use rayon::prelude::*;

use ::serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct XMLVersion {
    start_date: String,
    end_date: String,
    version_type: String,
}

impl XMLVersion {
    fn convert_to_version(&self) -> Version {
        let start_date: PrimitiveDateTime =
        PrimitiveDateTime::parse(&self.start_date, &Iso8601::DEFAULT).unwrap();
        let end_date: PrimitiveDateTime =
            PrimitiveDateTime::parse(&self.start_date, &Iso8601::DEFAULT).unwrap();

        return Version {
            start_date: start_date.date(),
            end_date: end_date.date(),
            version_type: self.version_type.clone(),
        };
    }
}
#[derive(Debug)]
pub struct Version {
    start_date: Date,
    end_date: Date,
    version_type: String,
}

#[derive(Debug)]
pub struct NeTExVersion {
    publication_time_stamp: PrimitiveDateTime,
    partition: String,
    netex_version: String,
    start_date: Date,
    end_date: Date,
    version_type: String,
}

pub fn parse_netex(s: String) -> Option<NeTExVersion> {
    let mut reader = Reader::from_str(&s);
    let mut junk_buf: Vec<u8> = Vec::new();
    let mut buf = Vec::new();

    let mut version = None;
    let mut netex_version = None;
    let mut partition = None;
    let mut publication_time_stamp = None;
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,
            Ok(Event::Empty(e)) => match e.name().as_ref() {
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
                    let version = netex_version_value.unwrap().unwrap().unescape_value().unwrap();
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
                    version = Some(xml_version.convert_to_version());
                },
                b"PublicationTimestamp" => {
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Text(e)) => {
                            publication_time_stamp =
                                Some(PrimitiveDateTime::parse(&e.unescape().unwrap(), &Iso8601::DEFAULT).unwrap());
                        }
                        _ => {}
                    }
                }
                _ => (),
            },
            _ => (),
        }
        if version.is_some() && netex_version.is_some() && partition.is_some() && publication_time_stamp.is_some() {
            let version = version.unwrap();
            return Some(NeTExVersion {
                publication_time_stamp: publication_time_stamp.unwrap(),
                partition: partition.unwrap(),
                netex_version: netex_version.unwrap(),
                start_date: version.start_date,
                end_date: version.end_date,
                version_type: version.version_type
            })
        }
        buf.clear();
    }
    return None;
}
