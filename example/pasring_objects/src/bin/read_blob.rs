
use std::fs::File;
use flate2::read::ZlibDecoder;
use std::io;
use std::io::prelude::*;
use std::path::Path;

pub fn main(){

  let path = std::env::args().nth(1).expect("no path given");

  read_blob(path);
      
}

fn read_blob(path: String){
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

  
  // let mut s: Vec<u8> = Vec::new();
  // z.read_to_end(&mut s)?;
  
  // let mut tmp:Vec<u8> = Vec::new();

  // let mut v:Vec<Vec<u8>> = Vec::new();
  // for i in s {
  //   tmp.push(i);
  //   if i==0u8 {
  //     v.push(tmp);
  //     tmp = Vec::new();
  //   } 
  // }
  // let mut result = String::new();

  // for n in 0..v.len(){
  //   let x = &v[n];
  //   if n == 0 || n == 1 {
  //     let str = std::str::from_utf8(&x).unwrap();
  //     result = result + str;
  //     println!("{}",result);
  //   } else {
  //     let sha = hex::encode(&v[n][..20]);
  //     // println!("{:}",sha);
  //     result = result + &sha;
  //     // println!("{:},",std::str::from_utf8(&v[n][20..]).unwrap());
  //     let str =std::str::from_utf8(&v[n][20..]).unwrap();
  //     result = result + str;

  //   }
  // }
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}