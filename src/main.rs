extern crate walkdir;

mod parse_netex;
mod xml_util;

use crate::parse_netex::parse_netex;
use std::io::Read;
use std::time::Instant;

use flate2::read::GzDecoder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

//use serde::Deserialize;

// use rayon::prelude::*;

fn main() {
    let start_time = Instant::now();

    println!("Hello, world!");
    println!("TEST!123");
    read_files();
    // Your task or code goes here
    // ...

    // Stop measuring time
    let end_time = Instant::now();

    // Calculate the duration
    let duration = end_time - start_time;

    // Print the duration in seconds and milliseconds
    println!(
        "Task duration: {} seconds and {} milliseconds",
        duration.as_secs(),
        duration.subsec_millis()
    );
}

fn unzip(file: &walkdir::DirEntry) -> String {
    //println!("{:?}", file.file_name());
    let bytes = std::fs::read(file.path()).unwrap();
    //println!("{}", file.metadata().unwrap().created().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());

    // println!("{:#?}", );
    let mut gz = GzDecoder::new(&bytes[..]);
    let mut s = String::new();
    gz.read_to_string(&mut s);
    let netex_version = parse_netex(s);
    if netex_version.is_some() {
        println!("{:?}", netex_version);
    }
    return "".to_string();
}

fn read_files() {
    let netex_files_to_process: Vec<walkdir::DirEntry> = WalkDir::new("./data")
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.metadata().unwrap().is_file())
        .collect();
    let _test: Vec<String> = netex_files_to_process
        .par_iter()
        .map(|file| unzip(file))
        .collect();

    println!("{}", netex_files_to_process.len())
}
