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

use gateway::pack::{index::Index, object};
use sqlx::mysql::{MySqlPoolOptions};
use gateway::database::mysql;

#[tokio::main]
async fn main() {
    // 设置 RUST_LOG，配合 tracing 打印日志
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "gateway=debug,tower_http=debug")
    }
    // 初始化 tracing
    tracing_subscriber::fmt::init();
    
    // 构建网关路由
    let app = Router::new()
        .route("/test.git/info/refs", get(handle_refs))
        .route("/test.git/git-receive-pack", post(process_pack));

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
/**
 * git push 过程中 client 与 server 有两个交互过程，第一步是引用发现，第二步是数据传输
 * handle_refs 是第一步
 */
async fn handle_refs(
    Query(service): Query<ServiceName>,
    _context: Request<Body>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    // Content-Type 需要是application/x-$servicename-advertisement，否则客户端会以哑协议的方式去处理
    let mode = format!("application/x-{}-advertisement", service.service);
    // Cache-Control 禁止缓存，不然可能看不到最新的提交信息
    headers.insert(
        "Cache-Control",
        "no-cache, max-age=0, must-revalidate".parse().unwrap(),
    );
    headers.insert("Content-Type", mode.parse().unwrap());
    headers.insert("Expires", "Fri, 01 Jan 1980 00:00:00 GMT".parse().unwrap());

    headers.insert("Pragma", "no-cache".parse().unwrap());

    if service.service != "git-receive-pack" && service.service != "git-upload-pack" {
        return (headers, String::from("Operation not permitted！！！"));
    }

    let repo_path = "/root/Tmp/repositories/test.git";
    
    let out = Command::new("git")
        .args(["init", "--bare", repo_path])
        .output()
        .expect("sh exec error!");

    tracing::debug!("create remote Git repository: {:?}", out.status);

    let path = Path::new(&repo_path);
    if !path.exists() {
        return (headers, String::from("Not Found!"));
    }
    let mut response_body = String::from("001f# service=git-receive-pack\n0000");
    let refs_bytes = Command::new("git") // git 数据检查
        .args([
            "receive-pack",
            "--stateless-rpc",
            "--advertise-refs",
            repo_path,
        ])
        .output()
        .expect("sh exec error!");

    let output = String::from_utf8(refs_bytes.stdout).unwrap();

    response_body = response_body + output.as_str(); // output 检查本地和服务器数据的不同 返回不同的引用

    (headers, response_body)
}
/**
 * 第二步数据传输
 */
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

    // 处理传输内容 
    read_body(&mut bytes).await.unwrap();

    let mut stdin = pipe.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(&mut bytes)
            .expect("Failed to write to stdin");
    });

    let output = pipe.wait_with_output().expect("Failed to read stdout");

    output.stdout
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
async fn read_body(body: &mut Bytes) -> Result<(), sqlx::Error> {
    let mut index = 0;
    if &body[index..index + 4] != b"0000" {
        let (context, len) = read_line(body, index);
        tracing::debug!("{}\n{}", context, len);
        index += len;
    }
    let mut packfile = body[index + 4..].to_vec();
    let offsets = Index::get_offset(&mut packfile).unwrap();
    
    // 连接数据库
    let database_url = "mysql://root:123456@localhost:3306/git";

    let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
    let mut conn = pool.acquire().await?;


    let mut len = offsets.len();

    // 逆序遍历 让数据写入数据库
    while len > 0 {
        len -= 1;
        let offset = offsets.get(len).unwrap();
        let object = object::read_object(&mut packfile, offset).unwrap();
        let mut git_index = mysql::GitIndex {
            sha_1: Some(object.hash.clone()),
            obj_type: object.meta_info.obj_type,
            size: object.meta_info.size,
            size_in_packfile: object.size_in_packfile,
            offset_in_pack: object.offset,
            depth: object.depth,
            base_sha_1: Some(object.base_sha_1.clone()),
        };
        mysql::insert(&mut git_index,&mut conn).await?;
        mysql::insert_blob(&mut object.hash.clone(), object.content.clone(), &mut conn).await?;
    }

    println!("end");

    Ok(())
}
// 按行读取
fn read_line(body: &mut Bytes, index: usize) -> (String, usize) {
    let line_len_str: String = format!("{:x?}", &body.slice(index..index + 4));

    let line_len = &hex::decode(&line_len_str[2..6]).unwrap();
    let len = line_len[0] as usize * 256 + line_len[1] as usize;
    if len == 0 {
        return ("".to_string(), 0);
    }
    let slice_context = &body.slice(index..len);

    let context = std::str::from_utf8(slice_context).unwrap();

    (String::from(context), len)
}