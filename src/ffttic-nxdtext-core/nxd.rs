use crate::{
    error::NxdError,
    nxd_tables::Cell,
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Seek, SeekFrom};


const NXD_MAGIC: u32 = u32::from_le_bytes(*b"NXDF");
const NXD_FORMAT: u32 = 1;


fn read_u32(reader: &mut impl ReadBytesExt) -> io::Result<u32> {
    reader.read_u32::<LittleEndian>()
}


fn read_cstr(reader: &mut impl ReadBytesExt) -> io::Result<String> {
    let mut buf = Vec::new();
    loop {
        match reader.read_u8()? {
            0 => break,
            c => buf.push(c),
        }
    }
    let text = String::from_utf8(buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(text)
}

fn read_cstr_at(reader: &mut (impl ReadBytesExt + Seek), offset: u64) -> io::Result<String> {
    let current_pos = reader.stream_position()?;
    reader.seek(SeekFrom::Start(offset))?;
    let text = read_cstr(reader)?;
    reader.seek(SeekFrom::Start(current_pos))?;
    Ok(text)
}


#[derive(Clone, Debug)]
pub struct Pointer {
    self_pos: u64,
    rel_offset: u32,
}

impl Pointer {
    pub fn read(reader: &mut (impl ReadBytesExt + Seek)) -> Result<Self, NxdError> {
        Ok(Self {
            self_pos: reader.stream_position()?,
            rel_offset: read_u32(reader)?,
        })
    }

    pub fn abs_target_from_self(&self) -> u64 {
        self.self_pos + (self.rel_offset as u64)
    }

    pub fn abs_target_from(&self, base: u64) -> u64 {
        base + (self.rel_offset as u64)
    }
}


#[derive(Clone, Debug)]
pub struct RowInfo {
    self_pos: u64,
    _row_key1: u32,
    _row_key2: Option<u32>,
    row_pos: Pointer,
}

impl RowInfo {
    pub fn read_1key(reader: &mut (impl ReadBytesExt + Seek)) -> Result<Self, NxdError> {
        Ok(Self {
            self_pos: reader.stream_position()?,
            _row_key1: read_u32(reader)?,
            _row_key2: None,
            row_pos: Pointer::read(reader)?,
        })
    }

    pub fn read_2key(reader: &mut (impl ReadBytesExt + Seek)) -> Result<Self, NxdError> {
        Ok(Self {
            self_pos: reader.stream_position()?,
            _row_key1: read_u32(reader)?,
            _row_key2: Some(read_u32(reader)?),
            row_pos: Pointer::read(reader)?,
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


#[derive(Clone, Debug)]
pub struct Header {}

impl Header {
    pub fn read_rowinfos(
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
        reader.seek(SeekFrom::Current(4 * 4))?;

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
}


fn read_cell(
    reader: &mut (impl ReadBytesExt + Seek),
    cell_type: &Cell,
) -> Result<Option<String>, NxdError> {
    fn safe_pos_add(base: u64, delta: i8) -> Result<u64, NxdError> {
        if delta >= 0 {
            base.checked_add(delta as _).ok_or(NxdError::InvalidHeader)
        } else {
            base.checked_sub(delta.unsigned_abs() as _).ok_or(NxdError::InvalidHeader)
        }
    }

    match cell_type {
        Cell::Zero32 | Cell::Bool32 | Cell::Skip32 | Cell::EmptyStr => {
            read_u32(reader)?;
            Ok(None)
        },
        Cell::Str(relative_field) => {
            let ptr = Pointer::read(reader)?;
            let ptr_base = safe_pos_add(ptr.self_pos, relative_field * 4)?;
            let text = read_cstr_at(reader, ptr.abs_target_from(ptr_base))?;
            Ok(Some(text))
        },
    }
}


pub fn read_row(
    reader: &mut (impl ReadBytesExt + Seek),
    row_definition: &[Cell],
    rowinfo: &RowInfo,
) -> Result<Vec<(usize, String)>, NxdError> {
    let row_pos = rowinfo.row_pos.abs_target_from(rowinfo.self_pos);
    reader.seek(SeekFrom::Start(row_pos))?;

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
