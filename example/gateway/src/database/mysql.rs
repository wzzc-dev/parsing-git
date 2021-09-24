// use crate::pack::packfile::Packfile;
// use mysql_async::prelude::*;

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct Index {
//     sha_1: Option<String>,
//     obj_type: u8,
//     size: usize,
//     size_in_packfile: usize,
//     offset_in_pack: usize,
//     depth: usize,
//     bash_sha_1: Option<String>,
// }

// pub async fn create_connect(packfile: Packfile) {
//     let database_url = "mysql://root:123456@localhost:3306/git";

//     let pool = mysql_async::Pool::new(database_url);

//     let conn = pool.get_conn().await;

//     // conn.query_drop(
//     //     r"create table `index1`
//     //     (
//     //         sha_1 char(40) not null,
//     //         obj_type int not null,
//     //         size int not null,
//     //         size_in_packfile int not null,
//     //         offset_in_pack int not null,
//     //         depth int null,
//     //         bash_sha_1 char(40) null
//     //     )"
//     // ).await;

//     for elem in packfile.objects {
//         match elem.meta_info.obj_type {
//             2 => {
//                 println!("hash:{}, context:{:?}", elem.hash, decode_tree(elem.data));
//                 let index  =  Index {
//                     sha_1: Some(elem.hash.into()),
//                     obj_type: elem.meta_info.obj_type,
//                     size: elem.meta_info.size as usize,
//                     size_in_packfile: elem.size_in_packfile as usize,
//                     offset_in_pack: elem.offset as usize,
//                     depth: 0,
//                     bash_sha_1: None,
//                 };
//                 let indexs = vec![index];
//         //         conn.exec_batch("INSERT INTO git (sha_1, obj_type, size, size_in_packfile, offset_in_pack, depth, bash_sha_1)
//         // VALUES (:sha_1, :obj_type, :size, :size_in_packfile, :offset_in_pack, :depth, :bash_sha_1)",indexs).await;

//             }
//             0..=4 => {
//                 println!(
//                     "hash:{}, context:{:?}",
//                     elem.hash,
//                     std::str::from_utf8(&elem.data)
//                 );
//             }
//             _ => {
//                 println!("hash:{}, context:{:?}", elem.hash, "def");
//             }
//         }
//     }

//     pool.disconnect().await;

// }


// fn decode_tree(s: Vec<u8>) -> String {
    
//     let mut result = String::new();
//     let mut offset = 0;
//     let mut tmp: Vec<u8> = Vec::new();
//     while offset < s.len() {
//         tmp.clear();
//         let f = offset;
//         for i in &s[f..] {
//             offset = offset + 1;
//             tmp.push(*i);
//             if *i == 0u8 {
//                 let name = std::str::from_utf8(&tmp).unwrap();
//                 let sha1 = &hex::encode(&s[offset..offset + 20]);
//                 result = result + name + " " + sha1 + "\n";
//                 offset = offset + 20;
//                 break;
//             }
//         }
//     }
//     result
// }