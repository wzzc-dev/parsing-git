use flate2::read::ZlibDecoder;
use crate::errors::{Result};
use std::io::Read;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tree {
    pub sha_1: Option<String>,
    pub name: Option<String>,
    pub content: Option<String>,
    pub file_type: Option<String>
}
impl Tree {
    pub fn new(pack:&mut Vec<u8>, offset: usize) -> String {
        let content = match decode_tree(pack[offset..].to_vec()) {
                Ok(v) => v,
                Err(_e) => "Invalid UTF-8 sequence".to_string(),
            };
        content
    }
    
}
fn decode_tree(bytes: Vec<u8>) -> Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s: Vec<u8> = Vec::new();
    z.read_to_end(&mut s)?;
  
    let mut result = String::new();
    let mut offset = 0;
    let mut tmp:Vec<u8> = Vec::new();
    
    while offset < s.len() {
      tmp.clear();
      let f = offset;
      for i in &s[f..] {
        offset = offset + 1;
        tmp.push(*i);
        if *i==0u8 {
          let name = std::str::from_utf8(&tmp).unwrap();
          let sha1 = &hex::encode(&s[offset..offset+20]);
          result = result + name + " " + sha1 + "\n";
          offset = offset + 20;
          break;
        }
      }
    }
  
    Ok(result)
  
  }