use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{self, Poll, Waker};

pub trait BatchLoader {
    type K: Hash + Eq + Clone;
    type V: Clone;

    async fn load_batch(&mut self, keys: Vec<Self::K>) -> HashMap<Self::K, Self::V>;
}

enum Entry<V> {
    Requested(Vec<Waker>),
    Ready(V),
}

struct LoaderInner<B: BatchLoader> {
    values: HashMap<B::K, Entry<B::V>>,
    pending_keys: HashMap<B::K, Vec<Waker>>,
    load_batch: B,
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
            values: Default::default(),
            pending_keys: Default::default(),
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn load(&self, key: B::K) -> impl Future<Output = B::V> {
        LoadFuture::Pending { loader: self, key }
    }

    pub fn wrap<O>(&self, fut: impl Future<Output = O>) -> impl Future<Output = O> {
        WrapFuture {
            loader: self,
            future: fut,
            currently_loading: None,
        }
    }
}

struct WrapFuture<'l, B: BatchLoader, F> {
    loader: &'l DataLoader<B>,
    future: F,
    currently_loading: Option<Pin<Box<dyn Future<Output = HashMap<B::K, B::V>> + Send + Sync>>>,
}

impl<'l, B: BatchLoader, F: Future> Future for WrapFuture<'l, B, F>
where
    B::K: Debug,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let slf = unsafe { self.get_unchecked_mut() };
        if let Some(currently_loading) = &mut slf.currently_loading {
            let currently_loading = currently_loading.as_mut();
            match currently_loading.poll(cx) {
                Poll::Ready(v) => {
                    let mut inner = slf.loader.inner.lock().unwrap();

                    for (k, v) in v {
                        if let Some(Entry::Requested(wakers)) =
                            inner.values.insert(k, Entry::Ready(v))
                        {
                            for w in wakers {
                                w.wake();
                            }
                        }
                    }

                    slf.currently_loading = None;
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        let future = unsafe { Pin::new_unchecked(&mut slf.future) };
        let res = future.poll(cx);
        if res.is_pending() {
            let mut inner = slf.loader.inner.lock().unwrap();

            if !inner.pending_keys.is_empty() {
                let mut keys = Vec::with_capacity(inner.pending_keys.len());
                for (k, v) in std::mem::take(&mut inner.pending_keys) {
                    keys.push(k.clone());
                    inner.values.insert(k, Entry::Requested(v));
                }

                let load_future = unsafe {
                    std::mem::transmute::<
                        Pin<Box<dyn Future<Output = _>>>,
                        Pin<Box<dyn Future<Output = _> + Send + Sync>>,
                    >(Box::pin(inner.load_batch.load_batch(keys)))
                };
                slf.currently_loading = Some(load_future);

                cx.waker().wake_by_ref();
            }
        }
        res
    }
}

enum LoadFuture<'l, B: BatchLoader> {
    Pending {
        loader: &'l DataLoader<B>,
        key: B::K,
    },
    Consumed,
}

impl<'l, B: BatchLoader> Future for LoadFuture<'l, B>
where
    B::K: Debug,
{
    type Output = B::V;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let slf = unsafe { self.get_unchecked_mut() };
        match slf {
            LoadFuture::Pending { loader, key } => {
                let mut inner = loader.inner.lock().unwrap();

                let wakers = match inner.values.get_mut(key) {
                    Some(Entry::Ready(v)) => {
                        *slf = LoadFuture::Consumed;
                        return Poll::Ready(v.clone());
                    }
                    Some(Entry::Requested(wakers)) => wakers,
                    None => inner.pending_keys.entry(key.clone()).or_insert_with(|| {
                        println!("starting to resolve related objects for `{key:?}`");
                        vec![]
                    }),
                };

                wakers.push(cx.waker().clone());
                Poll::Pending
            }
            LoadFuture::Consumed => panic!("`LoadFuture` polled after returning `Poll::Ready`"),
        }
    }
}
