use alloc::{string::String, sync::Arc};
use hashbrown::HashMap;

use crate::{arguments::Arguments, signature::Signature, Error, Value};
#[cfg(feature = "async")]
use core::future::Future;

#[derive(Clone)]
pub struct ServiceSignature<T: Value> {
    services: Arc<HashMap<String, Signature<T>>>,
}

impl<T: Value> From<HashMap<String, Signature<T>>> for ServiceSignature<T> {
    fn from(value: HashMap<String, Signature<T>>) -> Self {
        Self {
            services: Arc::new(value),
        }
    }
}

impl<T: Value> ServiceSignature<T> {
    pub fn iter(&self) -> hashbrown::hash_map::Iter<'_, String, Signature<T>> {
        self.services.iter()
    }

    pub fn functions(&self) -> hashbrown::hash_map::Keys<'_, String, Signature<T>> {
        self.services.keys()
    }

    pub fn get(&self, name: &str) -> Option<&Signature<T>> {
        self.services.get(name)
    }
}

pub trait Service<C, V: Value> {
    fn signature(&self) -> ServiceSignature<V>;
    // fn set_value(&self, name: &str, value: V) -> Result<(), Error<V>>;
    // fn get_value(&self, name: &str) -> Result<Option<V>, Error<V>>;
    fn call(&self, ctx: &mut C, name: &str, args: Arguments<V>) -> Result<V, Error<V>>;
}

#[cfg(feature = "async")]
pub trait AsyncService<C, V: Value> {
    // type Get<'a>: Future<Output = Result<Option<V>, Error<V>>>
    // where
    //     Self: 'a;
    // type Set<'a>: Future<Output = Result<(), Error<V>>>
    // where
    //     Self: 'a;
    type Call<'a>: Future<Output = Result<V, Error<V>>>
    where
        Self: 'a,
        C: 'a;

    fn signature(&self) -> ServiceSignature<V>;

    // fn set_value<'a>(&'a self, name: &'a str, value: V) -> Self::Set<'a>;
    // fn get_value<'a>(&'a self, name: &'a str) -> Self::Get<'a>;
    fn call<'a>(&'a self, ctx: &'a mut C, name: &'a str, args: Arguments<V>) -> Self::Call<'a>;
}

#[cfg(feature = "async")]
pub trait AsyncServiceExt<C, V: Value>: AsyncService<C, V> {
    fn boxed(self) -> super::BoxAsyncService<C, V>
    where
        C: 'static,
        V: Value + 'static,
        Self: AsyncService<C, V> + Send + Sync + 'static + Sized,
        // for<'a> Self::Set<'a>: Send,
        // for<'a> Self::Get<'a>: Send,
        for<'a> Self::Call<'a>: Send,
    {
        super::box_service(self)
    }
}

#[cfg(feature = "async")]
impl<T, C, V: Value> AsyncServiceExt<C, V> for T where T: AsyncService<C, V> {}
