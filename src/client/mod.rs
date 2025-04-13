use cynic::http::ReqwestExt;
use cynic::serde;
use reqwest::Url;

pub struct Client {
    client: reqwest::Client,
    url: Url,
}

impl Client {
    pub fn new(url: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
        }
    }

    pub async fn query<Query, Input>(
        &self,
        op: cynic::Operation<Query, Input>,
    ) -> cynic::GraphQlResponse<Query>
    where
        Input: serde::Serialize,
        Query: serde::de::DeserializeOwned + 'static,
    {
        self.client
            .post(self.url.clone())
            .run_graphql(op)
            .await
            .unwrap()
    }
}

#[cynic::schema("library")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
pub struct Library {
    pub authors: Vec<Author>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Author {
    pub name: String,
    pub born: String,
    pub books: Vec<Book>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Book {
    pub title: String,
    pub summary: String,
}
