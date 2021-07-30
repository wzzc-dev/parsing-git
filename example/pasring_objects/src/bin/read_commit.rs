
use std::fs::File;
use flate2::bufread::ZlibDecoder;
use std::io;
use std::io::prelude::*;
use std::path::Path;


pub fn main(){
    // let path = ".git/objects/a3/ae2f51f3ce58bae6e082e0ea138326a9b5be0f";
    // let path = ".git/objects/30/5157a396c6858705a9cb625bab219053264ee4";
    let path = ".git/objects/30/6c293c89a07c067bdf67320b0877e7820b2fb6";
    let file =  read_file(path.to_string());

    let file_content = decode_reader(file).unwrap();
    println!("{}",file_content);
    
    // let mut iter = file_content.split(|num|num%1 == 32);

    // let s = String::from_utf8(iter.next().unwrap().to_vec()).expect("Found invalid UTF-8");
   
    // println!("{:?}", s);
    // println!("{}", s);
     
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
    // let mut z = ZlibDecoder::new(&bytes[..]);
    // let mut s:Vec<u8> = Vec::new();
    // z.read_to_end(&mut s)?;
    // Ok(s)
}

// fn unpack_obj(raw: Vec<u8>){
//     raw.split(b' ')
// }