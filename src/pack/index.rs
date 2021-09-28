use flate2::read::ZlibDecoder;
use std::io::{Read};
use crate::errors::{ErrorKind, Result};

#[derive(Debug)]
pub struct Index {
    pub sha_1: Option<String>,
    pub obj_type: i8,
    pub size: i32,
    pub size_in_packfile: i32,
    pub offset_in_pack: i32,
    pub depth: i32,
    pub base_sha_1: Option<String>,
}
#[derive(Debug,Clone)]
pub struct Offset {
    pub size_in_packfile: i32,
    pub offset_in_pack: i32,
}
#[derive(Debug,Clone)]
pub struct MetaInfo {
    pub obj_type: u8,
    pub size: u64,
    pub consumed: u64,
}
impl Index {
    pub fn get_offset(pack:&mut Vec<u8>) -> Result<Vec<usize>> {
        // packfile header 解析
        let magic = &pack[0..4];

        if &magic != b"PACK" {
            return Err(ErrorKind::CorruptedPackfile.into());
        }

        let object_count = u32::from_be_bytes(to_array(pack[8..12].to_vec()));

        let mut i = 1;
        let mut offset: usize = 12;
        let mut offsets: Vec<usize> = Vec::new();
        offsets.push(12);

        while i < object_count {
            // packfile 对象解析
            let next_offset = next_offset(pack, &mut offset).unwrap();
            offsets.push(next_offset);
            i += 1;
            offset = next_offset;
        }

        Ok(offsets)
    }
}
use std::convert::TryInto;

fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

pub fn next_offset(pack: &mut Vec<u8>, offset: &mut usize) -> Result<usize> {
    let mut offset = *offset;

    let mut byte = pack[offset];
    offset += 1;

    let obj_type = (byte & 0x70) >> 4;
    let mut _size = (byte & 0xf) as u64;
    let mut consumed = 0;
    let mut continuation = byte & 0x80;
    loop {
        if continuation < 1 {
            break;
        }

        byte = pack[offset];
        offset += 1;
        continuation = byte & 0x80;

        _size |= ((byte & 0x7f) as u64) << (4 + 7 * consumed);
        consumed += 1;
    }

    match obj_type {
        0..=4 => { 
            // 1：commit; 2: tree; 3: blob; 4: tag
            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut data = Vec::new();
            deflate_stream.read_to_end(&mut data)?;
            offset += deflate_stream.total_in() as usize;
    
            return Ok(offset);
        },

        6 => {
            // OFS_DELTA 对象解析逻辑
            byte = pack[offset];
            offset += 1;
            let mut _negative_offset = u64::from(byte & 0x7F);

            while byte & 0x80 > 0 {
                _negative_offset += 1;
                _negative_offset <<= 7;
                byte = pack[offset];
                offset += 1;
                _negative_offset += u64::from(byte & 0x7F);
            }

            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut instructions = Vec::new();
            deflate_stream.read_to_end(&mut instructions)?;
            offset += deflate_stream.total_in() as usize;

            return Ok(offset);

        },

        7 => {
            // REF_DELTA 偏移
            offset += 20;

            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut instructions = Vec::new();
            deflate_stream.read_to_end(&mut instructions)?;
            offset += deflate_stream.total_in() as usize;
           

            return Ok(offset);

        },

        _ => Err(ErrorKind::BadLooseObject.into())
    }
}
