use crate::errors::{ErrorKind, Result};
use std::io::{BufRead, Seek, Write};
use flate2::read::ZlibDecoder;
use std::io::{Cursor, Read};

#[derive(Debug)]
pub struct Packfile {
    pub object_count: u32,
    pub version: u32,
    objects: Vec<Object<String>>
}
impl Packfile {
    pub fn new(mut pack: Vec<u8>) 
    // -> Result<Packfile> 
    {
        let mut offset = 0;
        let magic = &pack[0..4];
        // stream.read_exact(&mut magic)?;
        if &magic != b"PACK" {
            // return Err(ErrorKind::CorruptedPackfile.into());
        }
        // let mut version_bytes = [0u8; 4];
        // stream.read_exact(&mut version_bytes)?;
        let version = u32::from_be_bytes(to_array(pack[4..8].to_vec()));
        // match version {
        //     2 | 3 => (),
        //     _ => return Err(ErrorKind::NotImplemented.into()),
        // };
        // let mut object_count_bytes = [0u8; 4];
        // stream.read_exact(&mut object_count_bytes)?;
        let object_count = u32::from_be_bytes(to_array(pack[8..12].to_vec()));

        let mut i = 1;
        let mut offset:usize = 12;
        let mut objects:Vec<Object<String>> = Vec::new();
        while i < object_count {

            let object = packfile_read(&mut pack,&mut offset).unwrap();
            println!("{}, {:?}", offset, object);
            offset = (object.offset + object.size_in_packfile) as usize;


            objects.push(object);

            i += 1;

        }

        // Ok(Packfile {
        //     object_count,
        //     version,
        //     objects
        // })
    }
}

#[derive(Debug)]
pub struct Object <T>{
    pub meta_info: MetaInfo,
    pub offset: u64,
    pub size_in_packfile: u64,
    pub data: T,
}

#[derive(Debug)]
pub struct MetaInfo {
    pub obj_type: u8,
    pub size: u64,
    pub consumed: u64,
}


pub fn packfile_read(
    pack: &mut Vec<u8>,
    index:&mut usize
) -> Result<Object<String>> {
    let mut offset = *index;
    let start = offset;
    let mut byte = pack[offset];
    offset += 1;

    let obj_type = (byte & 0x70) >> 4;
    let mut size = (byte & 0xf) as u64;
    let mut consumed = 0;
    let mut continuation = byte & 0x80;
    loop {
        if continuation < 1 {
            break
        }

        let mut byte = pack[offset];
        offset += 1;
        continuation = byte & 0x80;

        size |= ((byte & 0x7f) as u64) << (4 + 7 * consumed);
        consumed += 1;
    }

    let meta_info = MetaInfo {obj_type, size, consumed};
    println!("{:?}", meta_info);

    let read_bytes;
    match obj_type {
        0..=4 => {
            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut s = String::new();
            deflate_stream.read_to_string(&mut s)?;
            offset += deflate_stream.total_in() as usize;
            read_bytes = 1 + consumed + deflate_stream.total_in();
            println!("{} , {} , {}",start, read_bytes, offset);
            return Ok(Object::<String>{
                meta_info: meta_info,
                offset: start as u64,
                size_in_packfile: read_bytes,
                data: s
            });
        },

        5 => { // OFS_DELTA
            let mut byte = pack[offset];
            offset += 1;
            let mut offset1 = u64::from(byte & 0x7F);

            while byte & 0x80 > 0 {
                offset1 += 1;
                offset1 <<= 7;
                let mut byte = pack[offset];
                offset += 1;
                offset1 += u64::from(byte & 0x7F);
                consumed += 1;
            }

            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut instructions = Vec::new();
            deflate_stream.read_to_end(&mut instructions)?;
            offset += deflate_stream.total_in() as usize;

            read_bytes = 2 + consumed + deflate_stream.total_in();
            return Ok(Object::<String>{
                meta_info: meta_info,
                offset: start as u64,
                size_in_packfile: read_bytes,
                data: "ofs unsupport".to_string()
            });
        },

        6 => { // REF_DELTA
            // let mut ref_bytes = [0u8; 20];
            // input.read_exact(&mut ref_bytes)?;
            // let id = Id::from(&ref_bytes);
            let mut ref_bytes = &pack[offset..offset+20];
            offset += 20;
            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut instructions = Vec::new();
            deflate_stream.read_to_end(&mut instructions)?;
            offset += deflate_stream.total_in() as usize;

            read_bytes = 21 + consumed + deflate_stream.total_in();
            return Ok(Object::<String>{
                meta_info: meta_info,
                offset: start as u64,
                size_in_packfile: read_bytes,
                data: "ref unsupport".to_string()
            });
        },

        _ => {
            Err(ErrorKind::BadLooseObject.into())
        }
    }
}
use std::convert::TryInto;

fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}