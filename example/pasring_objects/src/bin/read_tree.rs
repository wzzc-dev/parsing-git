
use std::fs::File;
use flate2::bufread::ZlibDecoder;
use std::io;
use std::io::prelude::*;
use std::path::Path;


pub fn main(){
    let path = ".git/objects/30/5157a396c6858705a9cb625bab219053264ee4";
    let file =  read_file(path.to_string());

    let file_content = decode_reader(file).unwrap();
    // println!("{:?}",file);
    // let mut z = ZlibDecoder::new(&file[..]);
    // let mut s:Vec<u8> = Vec::new();
    // println!("{:?}",z.read_to_end(&mut s));
    // println!("{:?}",s);
    println!("{:?}",std::str::from_utf8(&file_content[..23]).unwrap());
    let sha1 = &file_content[23..];
    println!("{:x?}", sha1);
    let sha = format!("{:x?}", sha1);
    println!("{}",sha);
    // let mut s = String::new();
    // println!("{:?}",z.read_to_string(&mut s));
    // println!("{:?}",std::str::from_utf8(s).unwrap())

    // println!("{:?}",file_content);
    // let s = &file_content[0..23];
    // println!("{:?}", std::str::from_utf8(s).unwrap());
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
fn decode_reader(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s:Vec<u8> = Vec::new();
    z.read_to_end(&mut s)?;
    Ok(s)
}
// fn decode_reader(bytes: Vec<u8>) -> io::Result<String> {
//     let mut z = ZlibDecoder::new(&bytes[..]);
//     let mut s = String::new();
//     z.read_to_string(&mut s)?;
//     Ok(s)
// }
// fn unpack_obj(raw: Vec<u8>){
//     raw.split(b' ')
// }