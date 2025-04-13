use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::Router;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::routing::get;

mod dataloader;
mod loaders;
mod schema;

use dataloader::DataLoader;
use loaders::{LoadBooks, LoadSummaries};
use schema::Library;

type FullSchema = Schema<Library, EmptyMutation, EmptySubscription>;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().finish())
}

async fn graphql_handler(State(schema): State<FullSchema>, req: GraphQLRequest) -> GraphQLResponse {
    let req = req
        .into_inner()
        .data(DataLoader::new(LoadBooks))
        .data(DataLoader::new(LoadSummaries));

    schema.execute(req).await.into()
}

pub fn make_app() -> Router {
    let schema = Schema::build(Library, EmptyMutation, EmptySubscription).finish();
    // std::fs::write("schemas/loader.graphql", schema.sdl()).unwrap();

    Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .with_state(schema)
}
