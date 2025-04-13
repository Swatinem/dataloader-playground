use std::collections::HashMap;

use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use tokio::task::yield_now;

use crate::datamodel::{ALL_AUTHORS, ALL_BOOKS};

use super::dataloader::{BatchLoader, DataLoader};
use super::loaders::{LoadBooks, LoadSummaries, Loaders as _};

pub struct Library;

#[Object]
impl Library {
    async fn authors(&self, _ctx: &Context<'_>) -> Vec<Author> {
        println!("starting to resolve authors");

        let authors = ALL_AUTHORS
            .iter()
            .map(|a| Author {
                name: a.name,
                born: a.born,
            })
            .collect();

        println!("finished resolving authors");
        authors
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Author {
    pub name: &'static str,
    pub born: &'static str,
}

#[ComplexObject]
impl Author {
    async fn books(&self, ctx: &Context<'_>) -> Vec<Book> {
        ctx.load_books(self.name).await
    }
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct Book {
    pub title: &'static str,
}

#[ComplexObject]
impl Book {
    async fn summary(&self, ctx: &Context<'_>) -> String {
        ctx.load_summary(self.title).await
    }
}
