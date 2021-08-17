use axum::{
    // async_trait,
    extract::{self, FromRequest, RequestParts,Query},
    prelude::*, 
    // response::{self, IntoResponse},
    // http::StatusCode
};
// use http::Response;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
// use tower::BoxError;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app =
        // route("/", post(git_receive_pack_2)).
        route("/info/refs", get(git_receive_pack_1));
        // .route("/info/refs?service=git-receive-pack", get(git_receive_pack));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize)]
struct ServiceName {
    service: String,
}

// async fn git_receive_pack(Query(service): Query<ServiceName>) -> String {
    
//     println!("{}", service.service);
//     service.service
// }
async fn git_receive_pack_1(Query(service): Query<ServiceName>, 
                            context: Request<Body>) -> &'static [u8] {
    println!("{:?}", service.service);
    println!("{:?}", context);
    
    "001f# service=git-receive-pack\n000000c000908\
    f76e437935be8f3afa4a6cb67315592b893 refs/heads/master \
    report-status report-status-v2 delete-refs side-band-64k \
    quiet atomic ofs-delta push-options object-format=sha1 agent=git/2.32.0\n0000".as_bytes()
}
async fn git_receive_pack_2(context: Request<Body>) -> String {
    println!("{:?}", context);
    "service.service".to_string()
}
