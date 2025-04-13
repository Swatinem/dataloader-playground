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

#[cynic::schema("loader")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
pub struct Child {
    pub id: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Parent {
    pub id: i32,
    pub children: Vec<Child>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "RootQuery")]
pub struct RootQuery {
    pub parents: Vec<Parent>,
}
