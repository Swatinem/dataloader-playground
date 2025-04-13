use std::net::{SocketAddr, TcpListener};

use axum::Router;
use reqwest::Url;

#[derive(Debug)]
pub struct Server {
    handle: tokio::task::JoinHandle<()>,
    socket: SocketAddr,
}

impl Server {
    pub fn with_router(router: Router) -> Self {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).unwrap();
        let socket = listener.local_addr().unwrap();

        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            axum::serve(listener, router).await.unwrap();
        });

        Self { handle, socket }
    }

    pub fn url(&self, path: &str) -> Url {
        let path = path.trim_start_matches('/');
        format!("http://localhost:{}/{}", self.socket.port(), path)
            .parse()
            .unwrap()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
