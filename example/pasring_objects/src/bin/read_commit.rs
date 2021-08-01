
use std::fs::File;
use flate2::read::ZlibDecoder;
use std::io;
use std::io::prelude::*;
use std::path::Path;

pub fn main(){

  let path = std::env::args().nth(1).expect("no path given");

  read_commit(path);
      
}

fn read_commit(path: String){
    let file = read_file(path);

    let file_content = decode_reader(file).unwrap();

    println!("{}", file_content); 
}

fn read_file(file_name: String) -> Vec<u8> {

    let path = Path::new(&file_name);
    if !path.exists() {
        return String::from("Not Found!").into();
    }
    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    file_content
}

fn decode_reader(bytes: Vec<u8>) -> io::Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}