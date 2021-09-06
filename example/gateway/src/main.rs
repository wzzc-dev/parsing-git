use axum::{
    body::{Body, Bytes},
    // async_trait,
    extract::Query,
    handler::{get, post},
    http::header::HeaderMap,
    http::Request,
    response::IntoResponse,
    Router,
};

use serde::Deserialize;
use std::net::SocketAddr;
use tower::BoxError;

use hex;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

use flate2::read::ZlibDecoder;
use std::io;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // route("/", post(git_receive_pack_2)).
        .route("/test.git/info/refs", get(handle_refs))
        .route("/test.git/git-receive-pack", post(process_pack));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct ServiceName {
    service: String,
}

// async fn git_receive_pack(Query(service): Query<ServiceName>) -> String {
//     println!("{}", service.service);
//     service.service
// }
async fn handle_refs(
    Query(service): Query<ServiceName>,
    _context: Request<Body>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let mode = format!("application/x-{}-advertisement", service.service);

    headers.insert(
        "Cache-Control",
        "no-cache, max-age=0, must-revalidate".parse().unwrap(),
    );
    headers.insert("Content-Type", mode.parse().unwrap());
    headers.insert("Expires", "Fri, 01 Jan 1980 00:00:00 GMT".parse().unwrap());

    headers.insert("Pragma", "no-cache".parse().unwrap());

    // println!("{:?}", service.service);
    // println!("{:?}", context.headers());
    if service.service != "git-receive-pack" && service.service != "git-upload-pack" {
        // return "Operation not permitted！！！".as_bytes();
        return (headers, String::from("Operation not permitted！！！"));
    }
    let repo_path = "/root/Tmp/repositories/test.git";
    Command::new("git")
        .args(["init", "--bare", repo_path])
        .output()
        .expect("sh exec error!");

    let path = Path::new(&repo_path);
    if !path.exists() {
        return (headers, String::from("Not Found!"));
    }
    let mut response_body = String::from("001f# service=git-receive-pack\n0000");
    let refs_bytes = Command::new("git") // 自己检查
        .args([
            "receive-pack",
            "--stateless-rpc",
            "--advertise-refs",
            repo_path,
        ])
        .output()
        .expect("sh exec error!");
    // println!("{:?}", refs_bytes);

    let output = String::from_utf8(refs_bytes.stdout).unwrap();

    response_body = response_body + output.as_str(); // output 检查本地和服务器数据的不同 返回不同的引用
                                                     // println!("{}", response_body);

    (headers, response_body)
}
async fn process_pack(req: Request<Body>) -> impl IntoResponse {
    let repo_path = "/root/Tmp/repositories/test.git";

    // 拦截 解析 pack
    let (_parts, body) = req.into_parts();
    let mut bytes = buffer_and_print("request", body).await.unwrap();

    // bytes
    let mut pipe = Command::new("git")
        .args(["receive-pack", "--stateless-rpc", repo_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    read_body(&mut bytes);

    let mut stdin = pipe.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(&mut bytes)
            .expect("Failed to write to stdin");
    });

    let output = pipe.wait_with_output().expect("Failed to read stdout");

    // // response
    // println!("{}", std::str::from_utf8(&output.stdout).unwrap());
    // println!("{:?}", std::str::from_utf8(&output.stdout));

    output.stdout
    // let response = "0023\u{2}Resolving deltas:   0% (0/74)\r0041\u{2}Resolving deltas:   1% (1/74)\rResolving deltas:   2% (2/74)\r0023\u{2}Resolving deltas:   4% (3/74)\r0023\u{2}Resolving deltas:   5% (4/74)\r0023\u{2}Resolving deltas:   6% (5/74)\r0023\u{2}Resolving deltas:   8% (6/74)\r0023\u{2}Resolving deltas:   9% (7/74)\r0023\u{2}Resolving deltas:  10% (8/74)\r0024\u{2}Resolving deltas:  13% (10/74)\r0024\u{2}Resolving deltas:  14% (11/74)\r0024\u{2}Resolving deltas:  16% (12/74)\r0043\u{2}Resolving deltas:  18% (14/74)\rResolving deltas:  20% (15/74)\r0085\u{2}Resolving deltas:  21% (16/74)\rResolving deltas:  24% (18/74)\rResolving deltas:  25% (19/74)\rResolving deltas:  27% (20/74)\rReso003f\u{2}lving deltas:  28% (21/74)\rResolving deltas:  31% (23/74)\r0024\u{2}Resolving deltas:  32% (24/74)\r0043\u{2}Resolving deltas:  35% (26/74)\rResolving deltas:  36% (27/74)\r0024\u{2}Resolving deltas:  37% (28/74)\r0024\u{2}Resolving deltas:  40% (30/74)\r0024\u{2}Resolving deltas:  41% (31/74)\r0024\u{2}Resolving deltas:  43% (32/74)\r0024\u{2}Resolving deltas:  44% (33/74)\r0024\u{2}Resolving deltas:  45% (34/74)\r0043\u{2}Resolving deltas:  47% (35/74)\rResolving deltas:  48% (36/74)\r0024\u{2}Resolving deltas:  50% (37/74)\r0024\u{2}Resolving deltas:  51% (38/74)\r0024\u{2}Resolving deltas:  52% (39/74)\r0024\u{2}Resolving deltas:  54% (40/74)\r0024\u{2}Resolving deltas:  55% (41/74)\r0024\u{2}Resolving deltas:  56% (42/74)\r0024\u{2}Resolving deltas:  58% (43/74)\r0024\u{2}Resolving deltas:  59% (44/74)\r0024\u{2}Resolving deltas:  60% (45/74)\r0024\u{2}Resolving deltas:  63% (47/74)\r0024\u{2}Resolving deltas:  64% (48/74)\r0024\u{2}Resolving deltas:  66% (49/74)\r0024\u{2}Resolving deltas:  67% (50/74)\r0043\u{2}Resolving deltas:  68% (51/74)\rResolving deltas:  70% (52/74)\r0043\u{2}Resolving deltas:  71% (53/74)\rResolving deltas:  72% (54/74)\r0024\u{2}Resolving deltas:  74% (55/74)\r0043\u{2}Resolving deltas:  75% (56/74)\rResolving deltas:  77% (57/74)\r0024\u{2}Resolving deltas:  79% (59/74)\r0024\u{2}Resolving deltas:  81% (60/74)\r0024\u{2}Resolving deltas:  82% (61/74)\r0024\u{2}Resolving deltas:  83% (62/74)\r0024\u{2}Resolving deltas:  85% (63/74)\r0024\u{2}Resolving deltas:  86% (64/74)\r0024\u{2}Resolving deltas:  87% (65/74)\r0024\u{2}Resolving deltas:  90% (67/74)\r0024\u{2}Resolving deltas:  91% (68/74)\r0024\u{2}Resolving deltas:  94% (70/74)\r0024\u{2}Resolving deltas:  95% (71/74)\r0024\u{2}Resolving deltas:  97% (72/74)\r0024\u{2}Resolving deltas:  98% (73/74)\r0024\u{2}Resolving deltas: 100% (74/74)\r002b\u{2}Resolving deltas: 100% (74/74), done.\n0030\u{1}000eunpack ok\n0019ok refs/heads/master\n00000000";
    // println!("{}", response);
    // response.as_bytes()
}
async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, BoxError>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: Into<BoxError>,
{
    let bytes = hyper::body::to_bytes(body).await.map_err(Into::into)?;
    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} body = {:?}", direction, body);
    }
    Ok(bytes)
}
fn read_body(body: &mut Bytes) {
    // let len = body.len();
    // println!("length: {}", len);
    // println!("{:?}", body.slice(0..4));
    let (context, len) = read_line(body, 0);
    println!("{}\n{}", context, len);
    println!("{:?}", read_line(body, len));

    println!("{:?}", read_packfile(body, len + 4))
}
fn read_line(body: &mut Bytes, index: usize) -> (String, usize) {
    let line_len_str: String = format!("{:x?}", &body.slice(index..index + 4));

    let line_len = &hex::decode(&line_len_str[2..6]).unwrap();
    let len = line_len[0] as usize * 256 + line_len[1] as usize;
    if len == 0 {
        return ("".to_string(), 0);
    }
    let slice_context = &body.slice(index..len);

    let context = std::str::from_utf8(slice_context).unwrap();

    // println!("{}", context);
    (String::from(context), len)
}
fn read_packfile(pack: &mut Bytes, index: usize) {
    // let pack:String = format!("{:x?}",&body.slice(index..index+4));
    // // let pack = &hex::decode(&line_len_str[2..6]).unwrap();

    // println!("{}", pack);

    // // println!("{:?}", &body.slice(index+4..index+100));
    // // let context = std::str::from_utf8(slice_context).unwrap();

    // let slice_context = &pack.slice(index+4..index+20);

    // let context = std::str::from_utf8(slice_context).unwrap();
    print!(
        "头部信息 12 字节：PACK:{:?}, ",
        std::str::from_utf8(&pack[index + 0..index + 4])
    );
    print!("版本号:{:?},", &pack[index + 4..index + 8]);
    // 条目的4字节号 n 个条目
    println!("条目数：{:?}", &pack[index + 8..index + 12]);
    let len = pack[index + 8] as usize * 256 * 256 * 256
        + pack[index + 9] as usize * 256 * 256
        + pack[index + 10] as usize * 256
        + pack[index + 11] as usize;
    // println!("{}", context);

    println!("{}", len);
    let mut offset = index + 12;
    // println!("{:?}", pack);
    let (mut type_id, mut size, mut consumed);
    let mut obj;

    loop{
        println!("{}", offset - index);

        let result = get_size(&pack[offset..]);
        type_id = result.0;
        size = result.1;
        consumed = result.2;
    
        println!("{:?}", (type_id, size, consumed));
        offset = offset + consumed as usize;
        let result = get_object(&mut pack[0..].to_vec() , offset ,offset + 1 + size as usize, type_id);
        obj = result.0;
        consumed = result.1 as usize;
        println!("{:?}, {}", obj, consumed);
        offset = offset + consumed as usize;
    }
    

    // for i in 0..len {
    //     let (type_id, _size, consumed) = get_size(&pack[offset..]);
    //     println!("{:?}", (type_id, _size, consumed));
    //     let mut s1: usize = pack.len();
    //     println!("{}", s1);
    //     // if i < len - 1 {
    //     //     s1 = index[i + 1].offset as usize;
    //     // }
    //     // println!("{:?}",index[i].sha1);
    //     // get_object(pack, offset + consumed, s1, type_id);
    //     // offset = s1
    // }

    println!("{:?}", "最后20位校检和:");
    println!("{:?}", &hex::encode(&pack[pack.len() - 20..pack.len()]));
    pack.clear();
}

fn get_size(data: &[u8]) -> (u8, u64, usize) {
    let mut c = data[0];
    let mut i = 1;
    let type_id = (c >> 4) & 0b0000_0111;
    let mut size = c as u64 & 0b0000_1111;
    let mut s = 4;
    while c & 0b1000_0000 != 0 {
        c = data[i];
        i += 1;
        size += ((c & 0b0111_1111) as u64) << s;
        s += 7
    }
    (type_id, size, i)
}
fn get_object(pack: &mut Vec<u8>, offset: usize, end: usize, obj_type: u8) -> (String, u64){
    
    if obj_type == 1 {
        decode_commit((&pack[offset..end]).to_vec())
    } else if obj_type == 2 {
        decode_tree((&pack[offset..end]).to_vec())
    } else if obj_type == 3 {
        decode_blob((&pack[offset..end]).to_vec())
    } else if obj_type == 4 {
        decode_tag((&pack[offset..end]).to_vec())
    } else  if obj_type == 5 {
        decode_ofs((&pack[offset..end]).to_vec())
    } else {
        ("".to_string(), 0)
    }
}
fn decode_ofs(bytes: Vec<u8>) -> (String, u64) {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s);
    (s,z.total_in())
}

fn decode_tag(bytes: Vec<u8>) -> (String, u64) {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s);
    (s,z.total_in())
}

fn decode_commit(bytes: Vec<u8>) -> (String, u64) {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s);
    (s,z.total_in())
}
fn decode_blob(bytes: Vec<u8>) ->  (String, u64) {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s);
    (s,z.total_in())
}

fn decode_tree(bytes: Vec<u8>) -> (String, u64) {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s: Vec<u8> = Vec::new();
    z.read_to_end(&mut s).expect("read tree error");
    let mut result = String::new();
    let mut offset = 0;
    let mut tmp: Vec<u8> = Vec::new();
    // for i in &s {
    //   offset = offset + 1;
    //   tmp.push(*i);
    //   if *i==0u8 {
    //     break;
    //   }
    // }
    // result = std::str::from_utf8(&tmp).unwrap().to_string() + "\n"; // 得到文件类型和大小
    while offset < s.len() {
        tmp.clear();
        let f = offset;
        for i in &s[f..] {
            offset = offset + 1;
            tmp.push(*i);
            if *i == 0u8 {
                let name = std::str::from_utf8(&tmp).unwrap();
                let sha1 = &hex::encode(&s[offset..offset + 20]);
                result = result + name + " " + sha1 + "\n";
                offset = offset + 20;
                break;
            }
        }
    }
    (result, z.total_in())
}
// fn read_object(input: &mut BufRead,read_bytes: &mut u64){
//     let output: &mut Write;
//     let mut byte = [0u8; 1];
//     input.read_exact(&mut byte);

//     let obj_type = (byte[0] & 0x70) >> 4;
//     let mut size = (byte[0] & 0xf) as u64;
//     let mut count = 0;
//     let mut continuation = byte[0] & 0x80;
//     loop {
//         if continuation < 1 {
//             break
//         }

//         input.read_exact(&mut byte);
//         continuation = byte[0] & 0x80;

//         size |= ((byte[0] & 0x7f) as u64) << (4 + 7 * count);
//         count += 1;
//     }

//     match obj_type {
//         0...4 => {
//             let mut deflate_stream = ZlibDecoder::new(input);
//             std::io::copy(&mut deflate_stream, output);
//             *read_bytes = 1 + count + deflate_stream.total_in();
//             return Ok(PackfileType::Plain(obj_type));
//         },

//         OFS_DELTA => {
//             input.read_exact(&mut byte)?;
//             let mut offset = u64::from(byte[0] & 0x7F);

//             while byte[0] & 0x80 > 0 {
//                 offset += 1;
//                 offset <<= 7;
//                 input.read_exact(&mut byte)?;
//                 offset += u64::from(byte[0] & 0x7F);
//                 count += 1;
//             }

//             let mut deflate_stream = ZlibDecoder::new(input);
//             let mut instructions = Vec::new();
//             deflate_stream.read_to_end(&mut instructions)?;

//             *read_bytes = 2 + count + deflate_stream.total_in();
//             return Ok(PackfileType::OffsetDelta((offset, instructions)))
//         },

//         REF_DELTA => {
//             let mut ref_bytes = [0u8; 20];
//             input.read_exact(&mut ref_bytes)?;
//             let id = Id::from(&ref_bytes);

//             let mut deflate_stream = ZlibDecoder::new(input);
//             let mut instructions = Vec::new();
//             deflate_stream.read_to_end(&mut instructions)?;
//             *read_bytes = 21 + count + deflate_stream.total_in();
//             return Ok(PackfileType::RefDelta((id, instructions)))
//         },

//         _ => {
//             Err(ErrorKind::BadLooseObject.into())
//         }
//     }
// }