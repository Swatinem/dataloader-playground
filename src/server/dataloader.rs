use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use std::task::Poll;

use tokio::sync::Mutex;
use tokio::task::yield_now;

pub trait BatchLoader {
    type K: Hash + Eq + Clone;
    type V: Clone;

    async fn load_batch(&mut self, keys: &[Self::K]) -> HashMap<Self::K, Self::V>;
}

struct LoaderInner<B: BatchLoader> {
    load_batch: B,
    requested_keys: Vec<B::K>,
    resolved_values: HashMap<B::K, B::V>,
}

pub struct DataLoader<B: BatchLoader> {
    inner: Arc<Mutex<LoaderInner<B>>>,
}
impl<B: BatchLoader> Clone for DataLoader<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<B: BatchLoader> DataLoader<B>
where
    B::K: Debug,
{
    pub fn new(load_batch: B) -> Self {
        let inner = LoaderInner {
            load_batch,
            requested_keys: Default::default(),
            resolved_values: Default::default(),
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub async fn load(&self, key: B::K) -> B::V {
        {
            let mut inner = self.inner.lock().await;
            println!("starting to resolve related objects for `{key:?}`");
            inner.requested_keys.push(key.clone());
        }
        yield_now().await;

        let mut inner = self.inner.lock().await;
        let requested_keys = std::mem::take(&mut inner.requested_keys);
        if !requested_keys.is_empty() {
            let resolved_values = inner.load_batch.load_batch(&requested_keys).await;
            inner.resolved_values.extend(resolved_values);
        }
        inner
            .resolved_values
            .get(&key)
            .expect("value should have been loaded")
            .clone()
    }
}
