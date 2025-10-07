use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Seek, SeekFrom};


pub fn read_u32(reader: &mut impl ReadBytesExt) -> io::Result<u32> {
    reader.read_u32::<LittleEndian>()
}
pub fn read_i32(reader: &mut impl ReadBytesExt) -> io::Result<i32> {
    reader.read_i32::<LittleEndian>()
}

pub fn write_u32(value: u32, writer: &mut impl WriteBytesExt) -> io::Result<()> {
    writer.write_u32::<LittleEndian>(value)
}


pub fn read_cstr(reader: &mut impl ReadBytesExt) -> io::Result<String> {
    let mut buf = Vec::new();
    loop {
        match reader.read_u8()? {
            0 => break,
            c => buf.push(c),
        }
    }
    let text = String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(text)
}

pub fn read_cstr_at(reader: &mut (impl ReadBytesExt + Seek), offset: u64) -> io::Result<String> {
    let current_pos = reader.stream_position()?;
    reader.seek(SeekFrom::Start(offset))?;
    let text = read_cstr(reader)?;
    reader.seek(SeekFrom::Start(current_pos))?;
    Ok(text)
}

pub fn write_cstr(text: &str, writer: &mut (impl WriteBytesExt + Seek)) -> io::Result<()> {
    writer.write_all(text.as_bytes())?;
    writer.write_u8(0x0)?;
    Ok(())
}
