
use std::fs::File;
// use flate2::read::ZlibDecoder;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use hex;


struct git_object {
  sha1: String,
  offset: u32,
}


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
  
  println!("decode_index");
  let mut idx = bytes;

  // println!("{:?}",&idx[4..1028]);
  // println!("{:}",&hex::encode(&idx[1028..1028+20]));


  // 索引文件头 8个字节 前四个字节总是255、116、79、99，后四个字节四个字节显式地表示版本号
  println!("索引文件头 8个字节 前四个字节总是255、116、79、99，后四个字节四个字节显式地表示版本号");
  println!("{:?}",&idx[0..8]);
  
  // 扇出表
  // 第一层 256个条目，每个条目4字节，总长度为1024字节
  println!("第一层 256个条目，每个条目4字节，总长度为1024字节");
  let mut n = 0;
  for i in (8..1032).filter(|x| ((x-8) % 4 == 0)) { // 一个条目告诉我们有多少（对象值）个对象以对象名（十六进制）开头
                                                // 第n个条目告诉我们n的值个(减去前面)对象以 十六进制的n 开头
    let m = &hex::encode(&idx[i..i+4]).parse::<usize>().unwrap();
    if m != &n {
      print!("{:x} *", (i - 8)/4);
      println!("{:?} ", m-n);
      n = *m;
    }

  }
  println!();
  // 第二层按顺序包含20字节的对象名称
  println!("第二层按顺序包含20字节的对象名称");
  let f = 1032 + 20 * n;
  for i in (1032..f).filter(|x| ((x-1032) % 20 == 0))  {
    let sha1 =  &hex::encode(&idx[i..i+20]);
    println!("{:?}",sha1);
    // objs.push(value: T)
  }

  println!();

  // 第三层 为每个对象提供了一个4字节的循环冗余校验
  println!("第三层 为每个对象提供了一个4字节的循环冗余校验");
  println!("{:?}",&hex::encode(&idx[f..f+4*n]));
  println!();

  // 第四层 每个对象的 packfile 偏移量 每个条目也有四个字节， 小于2gb MSB 为0，大于为1 偏移量存在第五层
  println!("第四层 每个对象的 packfile 偏移量 每个条目也有四个字节， 小于2gb MSB 为0，大于为1偏移量存在第五层");
  // println!("{:?}",&hex::encode(&idx[f+4*n..f+8*n]));
  for i in (f+4*n..f+8*n).filter(|x| ((x-(f+4*n)) % 4 == 0)) {
    // print!("{:x?} ",&hex::encode(&idx[i..i+4]));
    // println!("{:?}, ",&idx[i..i+4]);

    let offset23:u32 = idx[i+2].into();
    let offset01:u32 = idx[i+3].into();
    let offset:u32 = offset23*16*16+offset01;
    println!("{:?}",offset);

  }
  println!();
  // 代码优化 二四层合并
  let mut objs:Vec<git_object> = Vec::new();
  for i in 0..n{
    let sha1_index = 1032 + 20 * i;
    let offset_index = 1032 + 24 * n + 4 * i;

    let sha1 = &hex::encode(&idx[sha1_index..sha1_index+20]);
    let offset = idx[offset_index+2] as u32 * 16 * 16 + idx[offset_index+3] as u32;

    println!("{} {}", sha1, offset);

    let git_objects = git_object{
      sha1: String::from(sha1),
      offset : offset,
    };
    objs.push(git_objects);
  }


  // 第五层

  // 第六层 （packfile 的20字节校验和和整个索引文件的20字节校验和。）
  println!("第六层 （packfile 的20字节校验和和整个索引文件的20字节校验和。）");
  println!("{:?}",&hex::encode(&idx[idx.len()-40..idx.len()-20]));

  println!("{:?}",&hex::encode(&idx[idx.len()-20..idx.len()]));

  idx.clear();


  let s = String::new();
  Ok(s)
}

fn decode_pack(bytes: Vec<u8>) -> io::Result<String> {
  println!("decode_pack, -----------------------------------------");

  let mut pack = bytes;


  println!("{:?}",&pack[0..12]);

  pack.clear();


  let s = String::new();
  Ok(s)
}
