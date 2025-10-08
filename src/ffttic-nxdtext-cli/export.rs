use crate::{Error, path_to_tablename};
use ffttic_nxdtext_core as nxd;
use std::{
    fs::{self, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};


fn save_json(rows: &[(String, String)], out_path: &Path) -> Result<(), Error> {
    let mut map = serde_json::Map::with_capacity(rows.len());

    for (key, text) in rows {
        map.insert(key.clone(), serde_json::Value::String(text.clone()));
    }

    let json_content = serde_json::to_string_pretty(&map)?;

    let mut json_file = File::create(out_path)?;
    json_file.write_all(json_content.as_bytes())?;
    Ok(())
}


fn save_po(rows: &[(String, String)], out_path: &Path) -> Result<(), Error> {
    let mut catalog = polib::catalog::Catalog::new(Default::default());

    for (key, text) in rows {
        let message = polib::message::Message::build_singular()
            .with_msgctxt(key.clone())
            .with_msgid(text.clone())
            .done();
        catalog.append_or_update(message);
    }

    polib::po_file::write(&catalog, out_path)?;
    Ok(())
}


pub fn run(
    nxd_path: &Path,
    out_json: &Option<PathBuf>,
    out_po: &Option<PathBuf>,
) -> Result<(), Error> {
    let tablename = path_to_tablename(&nxd_path)?;

    let nxdfile = File::open(nxd_path)?;
    let mut reader = BufReader::new(nxdfile);
    let rows = nxd::read_rows(&mut reader, tablename)?;

    if let Some(json_path) = out_json {
        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent)?;
        }
        save_json(&rows, json_path)?;
    }
    if let Some(po_path) = out_po {
        if let Some(parent) = po_path.parent() {
            fs::create_dir_all(parent)?;
        }
        save_po(&rows, po_path)?;
    }

    Ok(())
}
