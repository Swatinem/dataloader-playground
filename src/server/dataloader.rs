use std::collections::HashMap;
use std::fmt::Debug;
use std::future::poll_fn;
use std::hash::Hash;
use std::marker::PhantomData;
use std::pin::{Pin, pin};
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
        poll_fn(move |cx| {
            let mut inner = self.inner.lock().unwrap();

            let wakers = match inner.values.get_mut(&key) {
                Some(Entry::Ready(v)) => {
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
        })
    }

    pub async fn wrap<O>(&self, fut: impl Future<Output = O>) -> O {
        // FIXME: Rust forces us to name this type explicitly, and we cannot use `impl Future` here
        // because that is not implemented yet (see <https://github.com/rust-lang/rust/issues/63065>).
        // We thus have to resort to a `dyn Future`, as we currently can’t name the future returned by `B::load()`
        // either, though something like "return type notation".
        let mut currently_loading: Option<
            Pin<Box<dyn Future<Output = HashMap<B::K, B::V>> + Send>>,
        > = None;

        let mut fut = pin!(fut);
        poll_fn(move |cx| {
            if let Some(currently_loading_fut) = &mut currently_loading {
                match currently_loading_fut.as_mut().poll(cx) {
                    Poll::Ready(v) => {
                        let mut inner = self.inner.lock().unwrap();

                        // Wake all the `load` calls waiting on this batch
                        for (k, v) in v {
                            if let Some(Entry::Requested(wakers)) =
                                inner.values.insert(k, Entry::Ready(v))
                            {
                                for w in wakers {
                                    w.wake();
                                }
                            }
                        }

                        currently_loading = None;
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            let res = fut.as_mut().poll(cx);
            if res.is_pending() {
                // We have polled the inner future once, during which it may have registered more
                // keys to load.
                let mut inner = self.inner.lock().unwrap();

                if !inner.pending_keys.is_empty() {
                    let mut keys = Vec::with_capacity(inner.pending_keys.len());
                    for (k, v) in std::mem::take(&mut inner.pending_keys) {
                        keys.push(k.clone());
                        inner.values.insert(k, Entry::Requested(v));
                    }

                    // FIXME: As have to resort to `dyn Future` for reasons explained above, and we have no way
                    // currently (without "return type notation") to require `B::load()` to return a `Send` future,
                    // we will just use an unsafe transmute, because YOLO.
                    let load_future: Pin<Box<dyn Future<Output = _> + Send>> = unsafe {
                        std::mem::transmute::<
                            Pin<Box<dyn Future<Output = _>>>,
                            Pin<Box<dyn Future<Output = _> + Send>>,
                        >(Box::pin(inner.load_batch.load_batch(keys)))
                    };
                    currently_loading = Some(load_future);

                    // Wake immediately, to instruct the runtime to call `poll` again.
                    cx.waker().wake_by_ref();
                }
            }
            res
        })
        .await
    }
}
