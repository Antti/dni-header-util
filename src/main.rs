use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use byteorder::{ReadBytesExt, WriteBytesExt};
use structopt::StructOpt;

mod dni_header;

#[derive(Debug, StructOpt)]
struct ShowCmd {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, StructOpt)]
struct SetCmd {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file
    #[structopt(parse(from_os_str))]
    output: PathBuf,

    #[structopt(name = "k", long = "key")]
    key: String,

    #[structopt(name = "v", long = "value")]
    value: String,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "dni-header-util",
    about = "Netgear DNI firmware header manipulation."
)]
enum Opt {
    Show(ShowCmd),
    Set(SetCmd),
}

fn calculate_checksum<T>(input: &mut T) -> io::Result<u8>
where
    T: Read,
{
    let mut last_byte = 0u8;
    let checksum = input.bytes().fold(0u8, |acc, byte| {
        last_byte = byte.unwrap();
        acc.overflowing_add(last_byte).0
    });
    Ok(0xFFu8 - (checksum - last_byte))
}

fn show_cmd(show_opts: &ShowCmd) -> io::Result<()> {
    let mut input_file = File::open(&show_opts.input)?;

    // read original checksum
    input_file.seek(io::SeekFrom::End(-1))?;
    let original_checksum = input_file.read_u8()?;

    // calculate real checksum
    input_file.seek(io::SeekFrom::Start(0))?;
    let mut buf_reader = io::BufReader::new(input_file);
    let checksum = calculate_checksum(&mut buf_reader)?;

    if checksum == original_checksum {
        println!("Checksum match: 0x{:X}", checksum);
    } else {
        eprintln!(
            "Warning: Expected checksum: 0x{:X}, got checksum: 0x{:X}",
            checksum, original_checksum
        );
    }

    buf_reader.seek(io::SeekFrom::Start(0))?;
    let header = dni_header::read(&mut buf_reader)?;
    println!("DNI Header:");
    for (k, v) in header {
        println!("  {}:{}", k, v);
    }
    Ok(())
}

fn set_cmd(set_opts: &SetCmd) -> io::Result<()> {
    let mut input = File::open(&set_opts.input)?;
    let mut output = File::create(&set_opts.output)?;

    println!("Writing header");
    let mut header = dni_header::read(&mut input)?;
    dni_header::insert(&mut header, set_opts.key.clone(), set_opts.value.clone());
    dni_header::write(&mut output, &header)?;
    input.seek(io::SeekFrom::Start(dni_header::DNI_HDR_LEN as u64))?;

    println!("Copying firmware");
    io::copy(&mut input, &mut output)?;
    output.flush()?;

    // opening read-only to calculate checksum:
    println!("Calculating checksum");
    let mut output = io::BufReader::new(File::open(&set_opts.output)?);
    output.seek(io::SeekFrom::Start(0))?;
    let checksum = calculate_checksum(&mut output)?;

    println!("Writing checksum: 0x{:X}", checksum);
    let mut output = OpenOptions::new().write(true).open(&set_opts.output)?;

    output.seek(io::SeekFrom::End(-1))?;
    output.write_u8(checksum)?;
    output.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    match opt {
        Opt::Show(cmd) => show_cmd(&cmd)?,
        Opt::Set(cmd) => set_cmd(&cmd)?,
    }
    Ok(())
}
