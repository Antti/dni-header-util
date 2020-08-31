use std::io;
use std::io::prelude::*;

pub const DNI_HDR_LEN: usize = 128usize;

pub type Header = Vec<(String, String)>;

pub fn read<T>(input: &mut T) -> io::Result<Header>
where
    T: Read,
{
    let buf = &mut [0u8; DNI_HDR_LEN];
    let _read_len = input.read(buf)?;

    let str_header = String::from_utf8_lossy(buf);
    let str_header = str_header.trim_matches('\u{0}');

    let fields: Vec<_> = str_header.split('\n').collect();
    let mut header = Header::new();
    for field in fields.iter() {
        if field.is_empty() {
            continue;
        }
        let name_and_value: Vec<&str> = field.split(':').collect();
        header.push((name_and_value[0].to_string(), name_and_value[1].to_string()));
    }
    Ok(header)
}

pub fn insert(header: &mut Header, key: String, value: String) {
    let pos = header.iter().position(|(k, _v)| k == &key);
    match pos {
        Some(pos) => header[pos] = (key, value),
        None => header.push((key, value)),
    }
}

#[allow(clippy::ptr_arg)]
pub fn write<T>(output: &mut T, header: &Header) -> io::Result<()>
where
    T: Write,
{
    let mut written_len = 0;
    for (key, value) in header {
        let field = format!("{}:{}\n", key, value);
        written_len += output.write(field.as_bytes())?;
    }
    assert!(written_len <= DNI_HDR_LEN);
    // padding
    output.write_all(&vec![0; DNI_HDR_LEN - written_len])?;
    Ok(())
}
