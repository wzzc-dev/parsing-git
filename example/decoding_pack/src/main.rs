
use std::fs::File;
// use flate2::read::ZlibDecoder;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use hex;

fn main() {
  let idx = std::env::args().nth(1).expect("no path given");

  let pack = std::env::args().nth(2).expect("no path given");

  read_packfile(idx,pack);
  
}

fn read_packfile(idx: String, pack: String){
  let idx = read_file(idx);

  let idx_content = decode_idx(idx).unwrap();
  println!("{:?}", idx_content);

  let pack = read_file(pack);
  let pack_content = decode_pack(pack).unwrap();
  println!("{:?}", pack_content);
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

// fn decode_packfile(bytes: Vec<u8>) -> io::Result<String> {

//   let mut idx = bytes;

//   println!("{:?}",&idx[0..4]);
//   // println!("{:?}",&idx[4..1028]);
//   // println!("{:}",&hex::encode(&idx[1028..1028+20]));

//   // println!("{:?}",&hex::encode(&idx[idx.len()-20..]));


//   println!("{:?}",&idx[0..12]);

//   idx.clear();


//   let s = String::new();
//   Ok(s)
// }

fn decode_idx(bytes: Vec<u8>) -> io::Result<String> {
  let mut idx = bytes;

  // println!("{:?}",&idx[4..1028]);
  // println!("{:}",&hex::encode(&idx[1028..1028+20]));



  println!("{:?}",&idx[0..8]);
  // println!("{:?}",&idx[1028..1032]);
  // println!("{:?}",&idx[8..1032]);

  // for n in 8..(1032-8)/4  {
  //   println!("{}",n);
  // }
  // println!("{:?}",&hex::encode(&idx[idx.len()-20..]));

  for i in (8..1032).filter(|x| (x % 4 == 0)) { // 一个条目告诉我们有多少（对象值）个对象以对象名（十六进制）开头
                                                // 第n个条目告诉我们n的值个(减去前面)对象以 十六进制的n 开头
                                                // 
    // print!("{} ", i);
    let m = &hex::encode(&idx[i..i+4]);
    // print!("{:?} ", m);
    if m == "00000003" {
      print!("{:x}   ", (i - 8)/4);
      println!("{:?} ", m);
    }

  }

  idx.clear();


  let s = String::new();
  Ok(s)
}

fn decode_pack(bytes: Vec<u8>) -> io::Result<String> {
  let mut pack = bytes;


  println!("{:?}",&pack[0..12]);

  pack.clear();


  let s = String::new();
  Ok(s)
}
