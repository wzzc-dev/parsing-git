use axum::{
    // async_trait,
    extract::{Query},
    http::{
        header::{HeaderMap},
    },
    handler::{post, get},
    response::{IntoResponse},
    http::{Request},
    body::{Body, Bytes},
    Router,
};

use serde::{Deserialize};
use std::net::SocketAddr;
use tower::{BoxError};

use std::path::Path;
use std::process::{Command, Stdio};
use std::io::prelude::*;

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
    
    // read_body(&mut bytes);

    // bytes
    let mut pipe = Command::new("git")
        .args(["receive-pack", "--stateless-rpc", repo_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    // read_body(&mut bytes);

    let mut stdin = pipe.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(&mut bytes)
            .expect("Failed to write to stdin");
    });

    let output =  pipe.wait_with_output().expect("Failed to read stdout");
    

    // response
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
// fn read_body(body: &mut Bytes){
//     println!("{:?}", body.slice(0..100));

// }