use flate2::bufread::ZlibDecoder;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use hex;

pub fn main() {

  let path = std::env::args().nth(1).expect("no path given");
  let file = read_file(path.to_string());

  let file_content = decode_reader(file).unwrap();  
  println!("{:?}",file_content);

  // println!("header :{:?},",std::str::from_utf8(&file_content[0..9]).unwrap());
  // print!("s:{:?},",std::str::from_utf8(&file_content[9..27]).unwrap());
  // println!("sha-1:{:x?},",&file_content[27..47]);
  
  // print!("s:{:?},",std::str::from_utf8(&file_content[47..62]).unwrap());
  // println!("sha-1:{:x?},",&file_content[62..82]);

  // println!("sha-1:{:x?},",&file_content[28..48]);

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
  let mut s: Vec<u8> = Vec::new();
  z.read_to_end(&mut s)?;
  
  let mut tmp:Vec<u8> = Vec::new();

  let mut v:Vec<Vec<u8>> = Vec::new();
  for i in s {
    tmp.push(i);
    if i==0u8 {
      v.push(tmp);
      tmp = Vec::new();
    } 
  }
  let mut result = String::new();

  for n in 0..v.len(){
    let x = &v[n];
    if n == 0 || n == 1 {
      let str = std::str::from_utf8(&x).unwrap();
      result = result + str;
    } else {
      let sha = hex::encode(&v[n][..20]);
      // println!("{:}",sha);
      result = result + &sha;
      // println!("{:},",std::str::from_utf8(&v[n][20..]).unwrap());
      let str =std::str::from_utf8(&v[n][20..]).unwrap();
      result = result + str;

    }

  }
  result = result + &hex::encode(&v[v.len()-1][..20]);
  Ok(result)
}
