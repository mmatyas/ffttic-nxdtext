use crate::{Error, path_to_tablename};
use ffttic_nxdtext_core as nxd;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
};


fn load_json(path: &Path, overrides: &mut HashMap<String, String>) -> Result<(), Error> {
    let file = File::open(path)?;
    let map: HashMap<String, String> = serde_json::from_reader(file)?;
    for (key, val) in map {
        overrides.insert(key, val);
    }
    Ok(())
}


fn load_po(path: &Path, overrides: &mut HashMap<String, String>) -> Result<(), Error> {
    let po_options = polib::po_file::POParseOptions {
        message_body_only: true,
        translated_only: true,
        unsafe_utf8_decode: false,
    };
    let catalog = polib::po_file::parse_with_option(path, &po_options)?;

    for message in catalog.messages() {
        if message.msgctxt().is_empty() {
            continue;
        }
        if let Ok(text) = message.msgstr() {
            overrides.insert(message.msgctxt().to_string(), text.to_string());
        }
    }
    Ok(())
}


pub fn run(
    nxd_path: &Path,
    in_json: &Option<PathBuf>,
    in_po: &Option<PathBuf>,
    out_nxd: &Path,
) -> Result<(), Error> {
    let tablename = path_to_tablename(&nxd_path)?;

    let mut text_overrides = HashMap::new();
    if let Some(json_path) = in_json {
        load_json(json_path, &mut text_overrides)?;
    }
    if let Some(po_path) = in_po {
        load_po(po_path, &mut text_overrides)?;
    }

    let nxdfile = File::open(nxd_path)?;
    let mut reader = BufReader::new(nxdfile);
    let out_buf = nxd::update_rows(&mut reader, tablename, &text_overrides)?;

    let mut out_file = File::create(out_nxd)?;
    out_file.write_all(&out_buf)?;

    Ok(())
}
