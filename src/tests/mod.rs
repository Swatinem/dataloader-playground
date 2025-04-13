use cynic::QueryBuilder as _;

use crate::client::{Client, Library};
use crate::server::make_app;

mod testserver;

#[tokio::test]
async fn test_request() {
    let app = make_app();
    let _server = testserver::Server::with_router(app);

    let client = Client::new(_server.url("/"));

    let query = Library::build(());
    let res = client.query(query).await.data;

    dbg!(res);
}
