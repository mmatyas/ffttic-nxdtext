use crate::{
    binary::*,
    error::NxdError,
    nxd_tables::{Cell, NXD_COLUMNS},
};
use byteorder::{ReadBytesExt};
use std::{
    collections::{HashMap, hash_map::Entry},
    io::{Cursor, Seek, SeekFrom, Write},
};


const NXD_MAGIC: u32 = u32::from_le_bytes(*b"NXDF");
const NXD_FORMAT: u32 = 1;


fn safe_pos_add(base: u64, delta: i32) -> Result<u64, NxdError> {
    let result = if delta.is_negative() {
        base.checked_sub(delta.unsigned_abs() as _)
    } else {
        base.checked_add(delta as _)
    };
    result.ok_or(NxdError::InvalidHeader)
}


#[derive(Clone, Debug)]
struct Pointer {
    self_pos: u64,
    rel_offset: i32,
}

impl Pointer {
    pub fn read(reader: &mut (impl ReadBytesExt + Seek)) -> Result<Self, NxdError> {
        Ok(Self {
            self_pos: reader.stream_position()?,
            rel_offset: read_i32(reader)?,
        })
    }

    pub fn abs_target_from(&self, base: u64) -> Result<u64, NxdError> {
        safe_pos_add(base, self.rel_offset as _)
    }
}


#[derive(Clone, Debug)]
struct RowInfo {
    self_pos: u64,
    _row_key1: u32,
    _row_key2: Option<u32>,
    rowdata_pos: Pointer,
}

impl RowInfo {
    pub fn read_1key(reader: &mut (impl ReadBytesExt + Seek)) -> Result<Self, NxdError> {
        Ok(Self {
            self_pos: reader.stream_position()?,
            _row_key1: read_u32(reader)?,
            _row_key2: None,
            rowdata_pos: Pointer::read(reader)?,
        })
    }

    pub fn read_2key(reader: &mut (impl ReadBytesExt + Seek)) -> Result<Self, NxdError> {
        Ok(Self {
            self_pos: reader.stream_position()?,
            _row_key1: read_u32(reader)?,
            _row_key2: Some(read_u32(reader)?),
            rowdata_pos: Pointer::read(reader)?,
        })
    }
}


fn read_key1_rowinfos(reader: &mut (impl ReadBytesExt + Seek)) -> Result<Vec<RowInfo>, NxdError> {
    let rowinfo_pos_abs = read_u32(reader)? as u64;
    let rowinfo_count = read_u32(reader)?;

    reader.seek(SeekFrom::Start(rowinfo_pos_abs))?;

    let rowinfos = (0..rowinfo_count)
        .map(|_| RowInfo::read_1key(reader))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rowinfos)
}

fn read_key2_rowinfos(reader: &mut (impl ReadBytesExt + Seek)) -> Result<Vec<RowInfo>, NxdError> {
    let _setinfo_pos = Pointer::read(reader)?;
    let _setinfo_count = read_u32(reader)?;
    let _blank = read_u32(reader)?;
    let rowinfo_pos_abs = read_u32(reader)? as u64;
    let rowinfo_count = read_u32(reader)?;

    reader.seek(SeekFrom::Start(rowinfo_pos_abs))?;

    let rowinfos = (0..rowinfo_count)
        .map(|_| RowInfo::read_2key(reader))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rowinfos)
}


enum NxdRowType {
    SingleKey = 1,
    DoubleKey,
}

#[derive(PartialEq)]
enum NxdLocalizationType {
    SingleKeyUnlocalized = 1,
    SingleKeyLocalized,
    DoubleKeyUnlocalized,
    DoubleKeyLocalized,
}


fn read_nxd_header(
    reader: &mut (impl ReadBytesExt + Seek),
) -> Result<Vec<RowInfo>, NxdError> {
    let magic = read_u32(reader)?;
    if magic != NXD_MAGIC {
        return Err(NxdError::InvalidHeader);
    }

    let format = read_u32(reader)?;
    if format != NXD_FORMAT {
        return Err(NxdError::InvalidHeader);
    }

    let table_rowtype = reader.read_u8()?;
    let table_localization = reader.read_u8()?;
    let _uses_base_rowid = reader.read_u8()?;
    let _blank = reader.read_u8()?;
    let _base_rowid = read_u32(reader);
    reader.seek_relative(4 * 4)?;

    match table_rowtype {
        f if f == NxdRowType::SingleKey as u8 => {
            let valid_localizations = &[
                NxdLocalizationType::SingleKeyUnlocalized as u8,
                NxdLocalizationType::SingleKeyLocalized as u8,
            ];
            if !valid_localizations.contains(&table_localization) {
                return Err(NxdError::InvalidHeader);
            }
            read_key1_rowinfos(reader)
        },
        f if f == NxdRowType::DoubleKey as u8 => {
            let valid_localizations = &[
                NxdLocalizationType::DoubleKeyUnlocalized as u8,
                NxdLocalizationType::DoubleKeyLocalized as u8,
            ];
            if !valid_localizations.contains(&table_localization) {
                return Err(NxdError::InvalidHeader);
            }
            read_key2_rowinfos(reader)
        },
        _ => Err(NxdError::UnsupportedFormat),
    }
}


fn read_cell(
    reader: &mut (impl ReadBytesExt + Seek),
    cell_type: &Cell,
) -> Result<Option<String>, NxdError> {
    match cell_type {
        Cell::Zero32 | Cell::Bool32 | Cell::Skip32 | Cell::EmptyStr => {
            read_u32(reader)?;
            Ok(None)
        },
        Cell::Str(relative_field) => {
            let ptr = Pointer::read(reader)?;
            let ptr_base = safe_pos_add(ptr.self_pos, (*relative_field as i32) * 4)?;
            let text_base = ptr.abs_target_from(ptr_base)?;
            let text = read_cstr_at(reader, text_base)?;
            Ok(Some(text))
        },
    }
}


fn read_row(
    reader: &mut (impl ReadBytesExt + Seek),
    row_definition: &[Cell],
    rowinfo: &RowInfo,
) -> Result<Vec<(usize, String)>, NxdError> {
    let rowdata_pos = rowinfo.rowdata_pos.abs_target_from(rowinfo.self_pos)?;
    reader.seek(SeekFrom::Start(rowdata_pos))?;

    let cells = row_definition
        .iter()
        .map(|cell_type| read_cell(reader, &cell_type))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .enumerate()
        .filter_map(|(idx, opt)| opt.map(|text| (idx, text)))
        .collect::<Vec<_>>();
    Ok(cells)
}


fn create_key(
    tablename: &str,
    row_idx: usize,
    cell_idx: usize,
) -> String {
    format!("{}/{}/{}", tablename, row_idx, cell_idx)
}


pub fn read_rows(
    reader: &mut (impl ReadBytesExt + Seek),
    tablename: &str,
) -> Result<Vec<(String, String)>, NxdError> {
    let row_definition = NXD_COLUMNS
        .get(tablename)
        .ok_or(NxdError::UnsupportedFormat)?;

    let rows = read_nxd_header(reader)?
        .into_iter()
        .map(|rowinfo| read_row(reader, &row_definition, &rowinfo))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .enumerate()
        .flat_map(|(row_idx, row)| row
            .into_iter()
            .map(move |(cell_idx, text)| {
                let key = create_key(tablename, row_idx, cell_idx);
                (key, text)
            })
        )
        .collect::<Vec<_>>();

    Ok(rows)
}


pub fn update_rows(
    reader: &mut (impl ReadBytesExt + Seek),
    tablename: &str,
    text_overrides: &HashMap<String, String>,
) -> Result<Vec<u8>, NxdError> {
    let row_definition = NXD_COLUMNS
        .get(tablename)
        .ok_or(NxdError::UnsupportedFormat)?;

    let (rowinfos, rows, textarea_abs_pos) = {
        let rowinfos = read_nxd_header(reader)?;
        let rowinfos_end = reader.stream_position()?;

        let rows = rowinfos
            .iter()
            .map(|rowinfo| read_row(reader, &row_definition, &rowinfo))
            .collect::<Result<Vec<_>, _>>()?;
        let rows_end = reader.stream_position()?;

        let textarea_abs_pos = std::cmp::max(rowinfos_end, rows_end);
        (rowinfos, rows, textarea_abs_pos)
    };

    let mut out_buf = {
        let capacity = reader.seek(SeekFrom::End(0))?;
        Cursor::new(Vec::with_capacity(capacity as _))
    };

    let mut tmp_buf = vec![0u8; textarea_abs_pos as _];
    reader.rewind()?;
    reader.read_exact(&mut tmp_buf)?;
    out_buf.write_all(&tmp_buf)?;

    let mut text_buf = Cursor::new(Vec::<u8>::new());
    let mut text_rel_offsets = HashMap::<String, u64>::new();

    if row_definition.contains(&Cell::EmptyStr) {
        let text = String::new();
        write_cstr(&text, &mut text_buf)?;
        text_rel_offsets.insert(text, 0);
    }

    for (row_idx, (rowinfo, rowdata)) in rowinfos.iter().zip(rows).enumerate() {
        let rowdata_pos = rowinfo.rowdata_pos.abs_target_from(rowinfo.self_pos)?;

        for (cell_idx, original_text) in rowdata {
            let cell_abs_pos = rowdata_pos + (cell_idx as u64) * 4;
            if cell_abs_pos >= textarea_abs_pos {
                return Err(NxdError::InvalidHeader);
            }
            out_buf.seek(SeekFrom::Start(cell_abs_pos))?;

            let key = create_key(tablename, row_idx, cell_idx);
            let text = text_overrides.get(&key).unwrap_or(&original_text);
            let text_abs_pos = {
                let text_rel_pos = match text_rel_offsets.entry(key) {
                    Entry::Occupied(slot) => slot.into_mut(),
                    Entry::Vacant(slot) => {
                        let pos = text_buf.stream_position()?;
                        write_cstr(&text, &mut text_buf)?;
                        slot.insert(pos)
                    },
                };
                textarea_abs_pos + *text_rel_pos
            };

            let ptr_base = {
                let relative_field = match row_definition[cell_idx] {
                    Cell::Str(shift) => shift,
                    _ => return Err(NxdError::InvalidHeader),
                };
                safe_pos_add(out_buf.stream_position()?, (relative_field as i32) * 4)?
            };

            let distance: u32 = text_abs_pos.checked_sub(ptr_base)
                .and_then(|val| val.try_into().ok())
                .ok_or(NxdError::InvalidHeader)?;
            write_u32(distance, &mut out_buf)?;
        }
    }

    out_buf.seek(SeekFrom::End(0))?;
    out_buf.write_all(&text_buf.into_inner())?;
    Ok(out_buf.into_inner())
}
