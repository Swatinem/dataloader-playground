#![allow(unused)]

use server::make_app;

mod client;
mod datamodel;
mod server;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {
    let app = make_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
