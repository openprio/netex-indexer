use std::collections::HashSet;

use rusqlite::{Connection, Result};

use crate::parse_netex::NeTExVersion;
extern crate time;
use time::macros::format_description;

pub fn insert(netex_files: Vec<NeTExVersion>) -> Result<()> {
    let conn = Connection::open("netex-index.db")?;

    for netex_file in netex_files.iter() {
        let valid_from = &netex_file
            .start_date
            .format(format_description!("[year]-[month]-[day]"))
            .unwrap();
        let valid_thru = &netex_file
            .end_date
            .format(format_description!("[year]-[month]-[day]"))
            .unwrap();
        let publication_timestamp = &netex_file
            .publication_time_stamp
            .format(format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second]"
            ))
            .unwrap();
        let _url = netex_file.path.split("data").nth(1).unwrap();
        let full_url = format!("https://data.ndovloket.nl/netex{}", _url);
        let res = conn.execute("
        INSERT INTO netex_version (partition, dataowner_code, file_name, valid_from, valid_thru, publication_timestamp, url, created_at) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))", (&netex_file.partition, &netex_file.data_owner_code, &netex_file.file_name, valid_from, valid_thru, publication_timestamp, full_url));
        if res.is_err() {
            println!("{}", res.unwrap_err());
        }
    }
    return Ok(());
}

pub fn init_db() -> Result<()> {
    let conn = Connection::open("netex-index.db")?;

    conn.execute(
        "create table if not exists netex_version (
            id integer primary key autoincrement,
            dataowner_code text not null,
            partition text not null,
            file_name text not null unique,
            valid_from text not null,
            valid_thru text not null,
            publication_timestamp text not null,
            url text not null,
            created_at text NOT NULL
        )",
        (),
    )?;

    return Ok(());
}

pub fn get_imported_files() -> Result<HashSet<String>> {
    let conn = Connection::open("netex-index.db")?;
    let mut result = HashSet::new();

    let mut q = conn.prepare(
        "
    SELECT file_name
    FROM netex_version",
    )?;
    let mut rows = q.query([])?;
    while let Some(row) = rows.next()? {
        let r: String = row.get(0)?;
        result.insert(r);
    }

    return Ok(result);
}
