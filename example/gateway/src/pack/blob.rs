use flate2::read::ZlibDecoder;
use crate::errors::{Result};
use std::io::Read;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Blob {
    pub sha_1: Option<String>,
    pub name: Option<String>,
    pub content: Option<String>,
    pub file_type: Option<String>
}
impl Blob {
    pub fn new(pack:&mut Vec<u8>, offset: usize) -> String {
        let content = match decode_blob(pack[offset..].to_vec()) {
                Ok(v) => v,
                Err(_e) => "Invalid UTF-8 sequence".to_string(),
            };
        content
    }
    
}
fn decode_blob(bytes: Vec<u8>) -> Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}