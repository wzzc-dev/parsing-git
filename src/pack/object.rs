use flate2::read::ZlibDecoder;
use std::io::{Read,Write};
use crypto::{digest::Digest, sha1::Sha1};
use crate::pack::delta::{DeltaDecoder, DeltaDecoderStream};
use crate::errors::{ErrorKind, Result};
use crate::pack::{blob,tree};

#[derive(Debug,Clone)]
pub struct Object {
    pub meta_info: MetaInfo,
    pub offset: u64,
    pub size_in_packfile: u64,
    pub hash: String,
    pub data: Vec<u8>,
    pub content: String,
    pub base_sha_1: String,
    pub depth: u64
}

#[derive(Debug,Clone)]
pub struct MetaInfo {
    pub obj_type: u8,
    pub size: u64,
    pub consumed: u64,
}
impl Object {
    
}
pub fn read_object(pack: &mut Vec<u8>, index: &usize) -> Result<Object> {
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
            break;
        }

        byte = pack[offset];
        offset += 1;
        continuation = byte & 0x80;

        size |= ((byte & 0x7f) as u64) << (4 + 7 * consumed);
        consumed += 1;
    }

    let meta_info = MetaInfo {
        obj_type,
        size,
        consumed,
    };

    match obj_type {
        0..=4 => { 
            // 1：commit; 2: tree; 3: blob; 4: tag
            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut data = Vec::new();
            deflate_stream.read_to_end(&mut data)?;
            offset += deflate_stream.total_in() as usize;
            let hash = get_hash(obj_type, &mut data).unwrap();
            let size_in_packfile = (offset - start) as u64;
            let content;
            if obj_type==2 {
                content = tree::Tree::new(pack, meta_info.consumed as usize + 1 +start);
            }else{
                content = blob::Blob::new(pack, meta_info.consumed as usize + 1 +start);
            }

            return Ok(Object {
                meta_info: meta_info,
                offset: start as u64,
                size_in_packfile,
                hash,
                data,
                content,
                base_sha_1: "".to_string(),
                depth: 0
            });
        },

        6 => {
            // OFS_DELTA 对象解析逻辑
            byte = pack[offset];
            offset += 1;
            let mut negative_offset = u64::from(byte & 0x7F);

            while byte & 0x80 > 0 {
                negative_offset += 1;
                negative_offset <<= 7;
                byte = pack[offset];
                offset += 1;
                negative_offset += u64::from(byte & 0x7F);

            }

            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut instructions = Vec::new();
            deflate_stream.read_to_end(&mut instructions)?;
            offset += deflate_stream.total_in() as usize;

            let mut ofs = start - negative_offset as usize;
            let ofs_object = read_object(pack, &mut ofs).unwrap();

            let (mut result, _written) = get_delta(ofs_object.data,&mut instructions);
            let content = blob::Blob::new(pack, meta_info.consumed as usize + 1 +start);

            return Ok(Object {
                meta_info: meta_info,
                offset: start as u64,
                size_in_packfile: (offset - start) as u64,
                hash: get_hash(obj_type, &mut result).unwrap(),
                data: result,
                base_sha_1: ofs_object.hash,
                content,
                depth: ofs_object.depth + 1
            });
        },

        7 => {
            // REF_DELTA 对象解析逻辑

            let ref_bytes = &pack[offset..offset + 20];
            offset += 20;

            let mut deflate_stream = ZlibDecoder::new(&pack[offset..]);
            let mut instructions = Vec::new();
            deflate_stream.read_to_end(&mut instructions)?;
            offset += deflate_stream.total_in() as usize;
           
            let content = "".to_string();

            return Ok(Object {
                meta_info: meta_info,
                offset: start as u64,
                size_in_packfile: (offset - start) as u64,
                hash: "".to_string(),
                data: instructions,
                content,
                base_sha_1: hex::encode(ref_bytes),
                depth: 0
            });
        },

        _ => Err(ErrorKind::BadLooseObject.into())
    }
}
// 对象 sha_1 计算
pub fn get_hash(object_type: u8, data: &mut Vec<u8>) -> Result<String> {
    let mut hash = Sha1::new();
    let mut header_buffer = Vec::new();
    write!(
        &mut header_buffer,
        "{} {}\0",
        as_str(object_type),
        data.len()
    )
    .ok();
    hash.input(&(header_buffer)[..]);
    hash.input(&(data)[..]);
    let mut id_output = [0u8; 20];
    hash.result(&mut id_output);
    Ok(hex::encode(id_output))
}
pub fn as_str(object_type: u8) -> &'static str {
    match object_type {
        1 => "commit",
        2 => "tree",
        3 => "blob",
        4 => "tag",
        _ => "blob",
    }
}
pub fn get_delta(base: Vec<u8>, instructions:&mut Vec<u8>) -> (Vec<u8>, usize) {
    let decoder = DeltaDecoder::new(&instructions, base).expect("wrong base size");
    let mut result = vec![0; decoder.output_size()];
    let mut decoder_stream: DeltaDecoderStream = decoder.into();

    let written = decoder_stream.read(&mut result).expect("read failed");

    (result, written)
}