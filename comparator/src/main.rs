use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::PathBuf,
};

use bytes::BytesMut;
use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg()]
    file_a: PathBuf,
    #[arg()]
    file_b: PathBuf,
}

fn main() {
    let args = Args::parse();
    let file_a = OpenOptions::new().read(true).open(args.file_a).unwrap();
    let file_b = OpenOptions::new().read(true).open(args.file_b).unwrap();
    let size = compare_files(file_a, file_b).unwrap();
    println!("Files are exactly the same and their size is {}", size);
}

const CMP_BUFFER: usize = 1024 * 1024;

fn compare_files(mut file_a: File, mut file_b: File) -> std::io::Result<usize> {
    let mut read = 0;
    let mut buff_a = BytesMut::zeroed(CMP_BUFFER);
    let mut buff_b = BytesMut::zeroed(CMP_BUFFER);
    loop {
        let read_a = file_a.read(&mut buff_a)?;
        let read_b = file_b.read(&mut buff_b)?;
        if read_a != read_b {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Files have different size!",
            ));
        }
        read += read_a;
        if buff_a != buff_b {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Files have different content!",
            ));
        }
        if read_a == 0 {
            break;
        }
    }
    Ok(read)
}
