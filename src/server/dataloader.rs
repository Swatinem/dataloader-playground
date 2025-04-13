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

struct LoaderInner<B> {
    load_batch: B,
}

pub struct DataLoader<B> {
    inner: Arc<Mutex<LoaderInner<B>>>,
}
impl<B> Clone for DataLoader<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<B> DataLoader<B>
where
    B: BatchLoader,
    B::K: Debug,
{
    pub fn new(load_batch: B) -> Self {
        let inner = LoaderInner { load_batch };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn load(&self, key: B::K) -> impl Future<Output = B::V> {
        async move {
            println!("starting to resolve related objects for `{key:?}`");
            yield_now().await;

            let mut inner = self.inner.lock().await;
            inner
                .load_batch
                .load_batch(&[key.clone()])
                .await
                .get(&key)
                .unwrap()
                .clone()
        }
        // LoadFuture::new(self, key)
    }
}

enum LoadFuture<'l, B>
where
    B: BatchLoader,
{
    Created {
        loader: &'l DataLoader<B>,
        key: B::K,
    },
    Pending {
        loader: &'l DataLoader<B>,
        key: B::K,
    },
    Ready(B::V),
}

impl<'l, B> LoadFuture<'l, B>
where
    B: BatchLoader,
{
    fn new(loader: &'l DataLoader<B>, key: B::K) -> Self {
        Self::Created { loader, key }
    }
}

impl<'l, B> Future for LoadFuture<'l, B>
where
    B: BatchLoader,
    B::K: Debug,
{
    type Output = B::V;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let n = std::any::type_name::<B::V>();
        todo!()
        // match self {
        //     LoadFuture::Created { loader, key } => {
        //         todo!();
        //         println!("starting to resolve `{N}` by `{key:?}`");
        //         *self = LoadFuture::Pending { loader, key };
        //         Poll::Pending
        //     }
        //     LoadFuture::Pending { loader, key } => todo!(),
        //     LoadFuture::Ready(v) => Poll::Ready(v),
        // }
    }
}
