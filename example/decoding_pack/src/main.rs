
use std::fs::File;
use flate2::read::ZlibDecoder;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use hex;


struct GitObject {
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

  let mut idx_content = decode_idx(idx).unwrap();
  // println!("{:?}", idx_content);
  // for elem in idx_content {
  //   println!("{},{}",elem.sha1,elem.offset);
  // }

  println!();
  let pack = read_file(pack);
  let pack_content = decode_pack(&mut idx_content ,pack).unwrap();
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

fn decode_idx(bytes: Vec<u8>) -> io::Result<Vec<GitObject>> {
  
  println!("decode_index");
  let idx = bytes;

  // 索引文件头 8个字节 前四个字节总是255、116、79、99，后四个字节四个字节显式地表示版本号
  // println!("索引文件头 8个字节 前四个字节总是255、116、79、99，后四个字节四个字节显式地表示版本号");
  // println!("{:?}",&idx[0..8]);
  
  // 扇出表
  // 第一层 256个条目，每个条目4字节，总长度为1024字节
  // println!("第一层 256个条目，每个条目4字节，总长度为1024字节");
  let mut n = 0;
  for i in (8..1032).filter(|x| ((x-8) % 4 == 0)) { // 一个条目告诉我们有多少（对象值）个对象以对象名（十六进制）开头
                                                // 第n个条目告诉我们n的值个(减去前面)对象以 十六进制的n 开头
    let m = &hex::encode(&idx[i..i+4]).parse::<usize>().unwrap();
    if m != &n {
      // print!("{:x} *", (i - 8)/4); // sha1码前两位
      // println!("{:?} ", m-n); // 有几个
      n = *m;
    }

  }

  // 第二层按顺序包含20字节的对象名称
  // println!("第二层按顺序包含20字节的对象名称");
  // let f = 1032 + 20 * n; // 第二层从1032到f
  // for i in (1032..f).filter(|x| ((x-1032) % 20 == 0))  {
  //   let sha1 =  &hex::encode(&idx[i..i+20]);
  //   println!("{:?}",sha1);
  // }

  // 第三层 为每个对象提供了一个4字节的循环冗余校验
  // println!("第三层 为每个对象提供了一个4字节的循环冗余校验");
  // println!("{:?}",&hex::encode(&idx[f..f+4*n])); 

  // 第四层 每个对象的 packfile 偏移量 每个条目也有四个字节， 小于2gb MSB 为0，大于为1 偏移量存在第五层
  // println!("第四层 每个对象的 packfile 偏移量 每个条目也有四个字节， 小于2gb MSB 为0，大于为1偏移量存在第五层");
  // println!("{:?}",&hex::encode(&idx[f+4*n..f+8*n]));
  // for i in (f+4*n..f+8*n).filter(|x| ((x-(f+4*n)) % 4 == 0)) {
  //   // print!("{:x?} ",&hex::encode(&idx[i..i+4]));
  //   // println!("{:?}, ",&idx[i..i+4]);

  //   let offset23:u32 = idx[i+2].into();
  //   let offset01:u32 = idx[i+3].into();
  //   let offset:u32 = offset23*16*16+offset01;
  //   println!("{:?}",offset)
  // }

  // 代码优化 二四层合并
  let mut objs:Vec<GitObject> = Vec::new();
  for i in 0..n{
    let sha1_index = 1032 + 20 * i;
    let offset_index = 1032 + 24 * n + 4 * i;

    let sha1 = &hex::encode(&idx[sha1_index..sha1_index+20]);
    let offset = idx[offset_index+2] as u32 * 16 * 16 + idx[offset_index+3] as u32;

    // println!("{} {}", sha1, offset);

    let git_objects = GitObject{
      sha1: String::from(sha1),
      offset : offset,
    };
    objs.push(git_objects);
  }


  // 第五层

  // 第六层 （packfile 的20字节校验和和整个索引文件的20字节校验和。）
  // println!("第六层 （packfile 的20字节校验和和整个索引文件的20字节校验和。）");
  // println!("{:?}",&hex::encode(&idx[idx.len()-40..idx.len()-20]));

  // println!("{:?}",&hex::encode(&idx[idx.len()-20..idx.len()]));

  Ok(objs)
}

fn decode_pack(index:&mut Vec<GitObject>, bytes: Vec<u8>) -> io::Result<String> {
  println!("decode_pack, -----------------------------------------");
  let mut pack = bytes;
  // PACK
  print!("头部信息 12 字节：PACK:{:?}, ",std::str::from_utf8(&pack[0..4]));
  // 版本号
  print!("版本号:{:?},",&pack[4..8]);
  // 条目的4字节号 n 个条目
  println!("条目数：{:?}",&pack[8..12]);

  // 对象头
  // if pack[12] > 128 { // 二进制 第一位为1
  //   let bin_idx = format!("{:b}", pack[13]) + &format!("{:b}", pack[12] << 4);
  //   println!("{}", &bin_idx[..bin_idx.len()-4]);
  //   let intval = isize::from_str_radix(&bin_idx[..bin_idx.len()-4], 2).unwrap();
  //   println!("{}", intval);
  // }
  // println!("{}",&format!("{:b}", pack[12]));
  // println!("{}",&format!("{:b}", pack[154]));
  println!("{}",&format!("{:b}", pack[262]));
  println!("{}",&format!("{:b}", pack[263]));
  println!("{}",&format!("{:b}", pack[264]));
  println!("{:?}",&pack[262..265]);
  // println!("{}",&format!("{:b}", pack[4210]));
  // println!("{}",&format!("{:b}", pack[4688]));
  println!("{:?}",pack.len());
  // println!("{:}",decode_reader((&pack[14..154]).to_vec()).unwrap());
  // // println!("{:?}", &format!("{:b}", pack[13]));
  // // println!("{:}",decode_reader((&pack[156..262]).to_vec()).unwrap());
  // println!("{:}",decode_reader((&pack[265..4210]).to_vec()).unwrap());
  // println!("{:}",decode_reader((&pack[4212..4688]).to_vec()).unwrap());
  // println!("{:}",decode_reader((&pack[4690..pack.len()-20]).to_vec()).unwrap());

  let len = index.len();
 
  quick_sort(index,0,len-1);

  // for elem in index {
  //   print!("{:?} ",elem.sha1);

  //   println!("{:?}",elem.offset);
  //   println!("{:}",decode_reader((&pack[14..154]).to_vec()).unwrap());

  // }
  // 读对象
//  println!("{}",len);
//  let mut offset = 12;
//   for i in 0..len{

//     if pack[offset] > 128 { // 二进制 第一位为1
//       let bin_idx = format!("{:b}", pack[offset+1]) + &format!("{:b}", pack[offset] << 4);
//       println!("{}", &bin_idx[..bin_idx.len()-4]);
//       let intval = isize::from_str_radix(&bin_idx[..bin_idx.len()-4], 2).unwrap();
//       println!("{}", intval);
//     }

//     let s1:usize = index[i+1].offset as usize;
//     println!("{:}",decode_reader((&pack[offset+2..s1]).to_vec()).unwrap());

//     offset = s1;
//   }
  // // 对象数
  // println!("{:?}", &pack[8..12]); // 不能超过 2^32
  // let num_objs:u32 = pack[8] as u32 *256*256*256 + pack[9] as u32 *256*256 + pack[10] as u32 *256 + pack[11] as u32; 
  // println!("{}", num_objs);

  // //
  println!("{:?}", "对象类型");
  println!("{:?}", pack[12] > 128);
  if pack[12] > 128 {
    let x = &format!("{:b}", pack[12]);
    println!("{}",x);
  }


  // println!("{}",pack[13]);

  // for elem in &pack[12..154] {
  //   println!("{}", elem);
  // }

  // println!("{:?}",  &pack[12..154]);
  // println!("{:?}", decode_reader((&pack[12..154]).to_vec()));

  // 最后20位校检和
  println!("{:?}", "最后20位校检和:");
  println!("{:?}", &hex::encode(&pack[pack.len()-20..pack.len()]));
  pack.clear();


  let s = String::new();
  Ok(s)
}

fn quick_sort(objects: &mut Vec<GitObject>, left: usize, right: usize) {
  if left >= right {
      return;
  }

  let mut l = left;
  let mut r = right;
  while l < r {
      while l < r && objects[r].offset >= objects[left].offset {
          r -= 1;
      }
      while l < r && objects[l].offset <= objects[left].offset {
          l += 1;
      }
      objects.swap(l, r);
  }
  objects.swap(left, l);
  if l > 1 {
      quick_sort(objects, left, l - 1);
  }

  quick_sort(objects, r + 1, right);

}


fn decode_reader(bytes: Vec<u8>) -> io::Result<String> {
  let mut z = ZlibDecoder::new(&bytes[..]);
  let mut s = String::new();
  z.read_to_string(&mut s)?;
  // println!("zzzzzzzzzzzzzzzzz{:?}",&bytes);
  // let mut v:Vec<u8> = Vec::new();
  // z.read_to_end(&mut v)?;
  // println!("zzzzzzzzzzzzz{:?}",v.len());
  Ok(s)
}