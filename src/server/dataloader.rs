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

    fn load_batch(
        &mut self,
        keys: Vec<Self::K>,
    ) -> impl Future<Output = HashMap<Self::K, Self::V>> + Send + 'static;
}

struct LoaderInner<B: BatchLoader> {
    load_batch: B,
    resolved_values: HashMap<B::K, B::V>,
    requested_keys: Vec<B::K>,
    pending_wakers: Vec<Waker>,
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

pub struct LoadFuture<'l, B: BatchLoader> {
    loader: &'l DataLoader<B>,
    key: B::K,
}

impl<'l, B: BatchLoader> Future for LoadFuture<'l, B>
where
    B::K: Debug,
{
    type Output = B::V;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.loader.inner.lock().unwrap();

        // Check the resolved value, and return it if it was resolved already
        if let Some(v) = inner.resolved_values.get(&self.key) {
            return Poll::Ready(v.clone());
        }

        // Otherwise, register the requested key, and its `Waker`
        println!("starting to resolve related objects for `{:?}`", self.key);
        inner.requested_keys.push(self.key.clone());
        inner.pending_wakers.push(cx.waker().clone());
        Poll::Pending
    }
}

impl<B: BatchLoader> DataLoader<B>
where
    B::K: Debug,
{
    pub fn new(load_batch: B) -> Self {
        let inner = LoaderInner {
            load_batch,
            resolved_values: Default::default(),
            requested_keys: Default::default(),
            pending_wakers: Default::default(),
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn load(&self, key: B::K) -> LoadFuture<B> {
        LoadFuture { loader: self, key }
    }

    pub async fn wrap<O>(&self, fut: impl Future<Output = O>) -> O {
        // FIXME: Rust forces us to name this type explicitly, and we cannot use `impl Future` here
        // because that is not implemented yet (see <https://github.com/rust-lang/rust/issues/63065>).
        // We thus have to resort to a `dyn Future`, as we currently can’t name the future returned by `B::load()`.
        let mut currently_loading: Option<
            Pin<Box<dyn Future<Output = HashMap<B::K, B::V>> + Send>>,
        > = None;

        let mut fut = pin!(fut);
        poll_fn(move |cx| {
            if let Some(currently_loading_fut) = &mut currently_loading {
                match currently_loading_fut.as_mut().poll(cx) {
                    Poll::Ready(resolved_values) => {
                        let mut inner = self.inner.lock().unwrap();

                        inner.resolved_values.extend(resolved_values);

                        // Wake all the `load` calls waiting on this batch
                        for waker in inner.pending_wakers.drain(..) {
                            waker.wake();
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

                let requested_keys = std::mem::take(&mut inner.requested_keys);
                if !requested_keys.is_empty() {
                    currently_loading = Some(Box::pin(inner.load_batch.load_batch(requested_keys)));

                    // Wake immediately, to instruct the runtime to call `poll` again right away.
                    cx.waker().wake_by_ref();
                }
            }
            res
        })
        .await
    }
}
