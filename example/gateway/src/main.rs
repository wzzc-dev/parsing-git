use axum::{
    // async_trait,
    extract::{self, FromRequest, Query, RequestParts},
    prelude::*,
    response::{self, IntoResponse},
    http::{
        self,
        header::{HeaderMap, HeaderValue},
        StatusCode,
    }
};
// use http::Response;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
// use tower::BoxError;
use std::path::Path;
use std::process::Command;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app =
        // route("/", post(git_receive_pack_2)).
        route("/test.git/info/refs", get(git_receive_pack_1))
        .route("/test.git/git-receive-pack", post(git_receive_pack_2));

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
async fn git_receive_pack_1(
    Query(service): Query<ServiceName>,
    context: Request<Body>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let mode = format!("application/x-{}-advertisement", service.service);

    headers.insert("Cache-Control", "no-cache, max-age=0, must-revalidate".parse().unwrap());
    headers.insert("Content-Type", mode.parse().unwrap());
    headers.insert("Expires", "Fri, 01 Jan 1980 00:00:00 GMT".parse().unwrap());

	headers.insert("Pragma", "no-cache".parse().unwrap());

    println!("{:?}", service.service);
    println!("{:?}", context);
    if service.service != "git-receive-pack" && service.service != "git-upload-pack" {
        // return "Operation not permitted！！！".as_bytes();
        return (headers, String::from("Operation not permitted！！！"))
    }
    let repo_path = "/root/Tmp/repositories/test.git";
    let path = Path::new(&repo_path);
    if !path.exists() {
        return (headers, String::from("Not Found!"));
    }
    let mut response_body = String::from("001f# service=git-receive-pack\n0000");
    let refs_bytes = Command::new("git").args(["receive-pack","--stateless-rpc", "--advertise-refs", repo_path])
    .output().expect("sh exec error!");
    println!("{:?}", refs_bytes);

    let output = String::from_utf8(refs_bytes.stdout).unwrap();
    println!("{}", output);

    response_body = response_body + output.as_str();

    // result
    // response_body
    // headers.append("Content-Type", "application/x-git-receive-pack-advertisement");
    // headers.insert

    (headers, response_body)

}
async fn git_receive_pack_2(context: Request<Body>) -> impl IntoResponse {
    println!("{:?}", context);
    (StatusCode::CREATED, "nothing to see here")
}
