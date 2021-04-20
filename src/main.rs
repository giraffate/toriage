mod pulls;

use hyper::{Body, Request, Response, Server};
use std::env;
use std::net::SocketAddr;
use tera::Tera;

use crate::pulls::PullsHandler;

#[derive(Clone, Debug)]
pub struct State {
    tera: Tera,
    token: String,
}

#[tokio::main]
async fn main() {
    let tera = Tera::new("templates/**/*").unwrap();
    let token = env::var("ACCESS_TOKEN").expect("personal access token is required");
    let pulls = PullsHandler { tera, token };

    let mut router = keiro::Router::new();
    router.get("/", index);
    router.get("/pulls/:owner/:repo", pulls);
    router.not_found(not_found);

    // let port = if let Some(port) = env::var("PORT").ok() {
    //     port
    // } else {
    //     "8080".to_string()
    // };
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    Server::bind(&addr)
        .serve(router.into_service())
        .await
        .unwrap();
}

async fn index(_: Request<Body>) -> keiro::Result<Response<Body>> {
    let response = Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(include_str!("../templates/index.html")))
        .unwrap();
    Ok(response)
}

async fn not_found(_req: Request<Body>) -> keiro::Result<Response<Body>> {
    let response = Response::builder()
        .status(404)
        .body(Body::from(include_str!("../templates/404.html")))
        .unwrap();
    Ok(response)
}
