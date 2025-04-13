use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use tokio::task::yield_now;

use crate::datamodel::{ALL_AUTHORS, ALL_BOOKS};

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
    name: &'static str,
    born: &'static str,
}

#[ComplexObject]
impl Author {
    async fn books(&self, _ctx: &Context<'_>) -> Vec<Book> {
        println!("starting to resolve Books by `{}`", self.name);
        yield_now().await;
        println!("actually resolving Books by `{}`", self.name);

        let books = ALL_BOOKS
            .iter()
            .filter(|b| b.author == self.name)
            .map(|b| Book { title: b.title })
            .collect();

        println!("finished resolving Books by `{}`", self.name);
        books
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Book {
    title: &'static str,
}

#[ComplexObject]
impl Book {
    async fn summary(&self, _ctx: &Context<'_>) -> String {
        println!("starting to resolve summary for `{}`", self.title);
        yield_now().await;
        println!("actually resolving summary for `{}`", self.title);

        let summary = "Don’t know, read it yourself!".into();

        println!("finished resolving summary for `{}`", self.title);
        summary
    }
}
