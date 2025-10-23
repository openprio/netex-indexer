extern crate walkdir;

mod db;
mod parse_netex;
mod xml_util;

use crate::{db::get_imported_files, parse_netex::parse_netex};
use std::io::Read;
use std::time::Instant;

use db::init_db;
use flate2::read::GzDecoder;
use parse_netex::NeTExVersion;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

fn main() {
    let start_time = Instant::now();

    println!("Start NeTEx indexer");
    init_db();
    let result = get_imported_files();
    let netex_files = read_files();
    db::insert(netex_files);

    // Your task or code goes here
    // ...

    // Stop measuring time
    let end_time = Instant::now();

    // Calculate the duration
    let duration = end_time - start_time;

    // Print the duration in seconds and milliseconds
    println!(
        "Import took: {} seconds and {} milliseconds",
        duration.as_secs(),
        duration.subsec_millis()
    );
}

fn unzip(file: &walkdir::DirEntry) -> Option<NeTExVersion> {
    let bytes = std::fs::read(file.path()).unwrap();

    let mut gz = GzDecoder::new(&bytes[..]);
    let mut s = String::new();
    let _ = gz.read_to_string(&mut s);
    return parse_netex(
        s,
        file.file_name().to_str().unwrap().to_string(),
        file.path().to_str().unwrap().to_string(),
    );
}

fn is_not_excluded(entry: &walkdir::DirEntry) -> bool {
    let excluded_dirs = vec!["epiap", "enum", "test"]; // Directories to exclude

    println!("{:?}", entry.path().as_os_str().to_str());
    // Check if the current entry's path matches any of the excluded directories
    for excluded in excluded_dirs {
        if entry.path().parent().is_some_and(|f| f.ends_with(excluded)) {
            return false;
        }
    }

    let excluded_files = vec!["vehicle"]; // Files to exclude

    for excluded in excluded_files {
        if entry.file_name().to_str().unwrap().contains(excluded) {
            return false;
        }
    }

    true
}

fn read_files() -> Vec<NeTExVersion> {
    let netex_files_to_process: Vec<walkdir::DirEntry> = WalkDir::new("./data")
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.metadata().unwrap().is_file() && is_not_excluded(file))
        .collect();
    let mut result: Vec<Option<NeTExVersion>> = netex_files_to_process
        .par_iter()
        .map(|file| unzip(file))
        .filter(|x| x.is_some() && x.as_ref().unwrap().version_type == "baseline")
        .collect();

    println!("{}", result.len());
    //result.iter().for_each(|f| println!("{}", f.as_ref())
    result.sort_by_key(|f| f.as_ref().unwrap().publication_time_stamp);
    result
        .iter()
        .for_each(|f| println!("{}", f.as_ref().unwrap().publication_time_stamp));

    result.iter().for_each(|f| {
        if f.is_some() {
            println!("{:?}", f);
        }
    });

    println!("{}", netex_files_to_process.len());
    let res: Vec<NeTExVersion> = result
        .iter()
        .filter(|netex_file| netex_file.is_some())
        .map(|f| f.clone().unwrap())
        .collect();
    return res;
}
