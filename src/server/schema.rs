use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use tokio::task::yield_now;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Parent {
    id: usize,
}

#[ComplexObject]
impl Parent {
    async fn children(&self, _ctx: &Context<'_>) -> Vec<Child> {
        println!("resolving children of {}", self.id);
        yield_now().await;
        println!("finished resolving children of {}", self.id);

        vec![]
    }
}

#[derive(SimpleObject)]
pub struct Child {
    id: usize,
}

pub struct RootQuery;

#[Object]
impl RootQuery {
    async fn parents(&self, _ctx: &Context<'_>) -> Vec<Parent> {
        (0..4).map(|id| Parent { id }).collect()
    }
}
