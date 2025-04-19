use std::collections::HashMap;

use async_graphql::Context;

use crate::datamodel::ALL_BOOKS;

use super::dataloader::{BatchLoader, DataLoader};
use super::schema::Book;

pub trait Loaders {
    async fn load_books(&self, name: &'static str) -> Vec<Book>;
    async fn load_summary(&self, title: &'static str) -> String;
}

impl Loaders for Context<'_> {
    async fn load_books(&self, name: &'static str) -> Vec<Book> {
        self.data_unchecked::<DataLoader<LoadBooks>>()
            .load(name)
            .await
    }

    async fn load_summary(&self, title: &'static str) -> String {
        self.data_unchecked::<DataLoader<LoadSummaries>>()
            .load(title)
            .await
    }
}

pub struct LoadBooks;
impl BatchLoader for LoadBooks {
    type K = &'static str;
    type V = Vec<Book>;

    fn load_batch(
        &mut self,
        keys: Vec<Self::K>,
    ) -> impl Future<Output = HashMap<Self::K, Self::V>> + Send + 'static {
        async move {
            let debug_keys = keys.join("`, `");
            println!("actually resolving Books by `{debug_keys}`");

            let mut books: HashMap<_, Vec<Book>> = HashMap::with_capacity(keys.len());
            for book in ALL_BOOKS {
                if keys.contains(&book.author) {
                    books
                        .entry(book.author)
                        .or_default()
                        .push(Book { title: book.title });
                }
            }

            println!("finished resolving Books by `{debug_keys}`");
            books
        }
    }
}

pub struct LoadSummaries;
impl BatchLoader for LoadSummaries {
    type K = &'static str;
    type V = String;

    fn load_batch(
        &mut self,
        keys: Vec<Self::K>,
    ) -> impl Future<Output = HashMap<Self::K, Self::V>> + Send + 'static {
        async move {
            let debug_keys = keys.join("`, `");
            println!("actually resolving summaries for `{debug_keys}`");

            let summaries = keys
                .iter()
                .map(|k| (*k, "Don’t know, read it yourself!".into()))
                .collect();

            println!("finished resolving summaries for `{debug_keys}`");
            summaries
        }
    }
}
