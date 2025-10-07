use crate::{Error, path_to_tablename};
use ffttic_nxdtext_core::{
    nxd,
    NXD_COLUMNS,
};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};


fn save_json(
    tablename: &str,
    rows: &[(usize, usize, String)],
    out_path: &Path,
) -> Result<(), Error> {
    let mut map = serde_json::Map::with_capacity(rows.len());

    for (row_idx, cell_idx, text) in rows {
        let key = format!("{}/{}/{}", tablename, row_idx, cell_idx);
        map.insert(key, serde_json::Value::String(text.clone()));
    }

    let json_content = serde_json::to_string_pretty(&map)?;

    let mut json_file = File::create(out_path)?;
    json_file.write_all(json_content.as_bytes())?;
    Ok(())
}


fn save_po(
    tablename: &str,
    rows: &[(usize, usize, String)],
    out_path: &Path,
) -> Result<(), Error> {
    let mut catalog = polib::catalog::Catalog::new(Default::default());

    for (row_idx, cell_idx, text) in rows {
        let key = format!("{}/{}/{}", tablename, row_idx, cell_idx);
        let message = polib::message::Message::build_singular()
            .with_msgctxt(key)
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
    let row_definition = NXD_COLUMNS
        .get(tablename)
        .ok_or(format!("Unknown or unsupported table name `{}`", tablename))?;

    let nxdfile = File::open(nxd_path)?;
    let mut reader = BufReader::new(nxdfile);

    let rows = nxd::Header::read_rowinfos(&mut reader)?
        .into_iter()
        .map(|rowinfo| nxd::read_row(&mut reader, &row_definition, &rowinfo))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .enumerate()
        .flat_map(|(row_idx, row)| row
            .into_iter()
            .map(move |(cell_idx, text)| (row_idx, cell_idx, text))
        )
        .collect::<Vec<_>>();

    if let Some(json_path) = out_json {
        save_json(tablename, &rows, json_path)?;
    }
    if let Some(po_path) = out_po {
        save_po(tablename, &rows, po_path)?;
    }

    Ok(())
}
