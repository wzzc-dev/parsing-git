use flate2::bufread::ZlibDecoder;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

pub fn main() {
  let path = std::env::args().nth(1).expect("no path given");
  let file = read_file(path.to_string());

  let file_content = decode_reader(file).unwrap();
  print!("{:?}",file_content);
  // println!("{:?}", std::str::from_utf8(&file_content[..27]).unwrap());

  let mut s : Vec<u8> = Vec::new();
  let mut i = 0;
  for n in file_content {
    
    if  n == 0  {
      println!("{:?},",std::str::from_utf8(&s).unwrap());
      s.clear();
      i = i + 1;
    } else {
      print!("{:?} ",n);
      s.push(n);
      println!("---------------{:?}--- ",i);
      if i == 2 && s.len() == 40 * 4 {
        s.clear();
      }
    }
  }

  // let mut iter = file_content.split(|num|num%1 == 32);
  // for elem in iter {
  //   let s = String::from_utf8(elem.to_vec()).expect("Found invalid UTF-8");
   
  //   println!("{:?}", s);
  // }
  
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
  let mut s: Vec<u8> = Vec::new();
  z.read_to_end(&mut s)?;
  Ok(s)
}
