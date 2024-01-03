use crate::{arguments::Arguments, Error, Value};
#[cfg(feature = "async")]
use core::future::Future;
pub trait Service<C, V: Value> {
    fn set_value(&self, name: &str, value: V) -> Result<(), Error<V>>;
    fn get_value(&self, name: &str) -> Result<Option<V>, Error<V>>;
    fn call(&self, ctx: &mut C, name: &str, args: Arguments<V>) -> Result<V, Error<V>>;
}

#[cfg(feature = "async")]
pub trait AsyncService<C, V: Value> {
    type Get<'a>: Future<Output = Result<Option<V>, Error<V>>>
    where
        Self: 'a;
    type Set<'a>: Future<Output = Result<(), Error<V>>>
    where
        Self: 'a;
    type Call<'a>: Future<Output = Result<V, Error<V>>>
    where
        Self: 'a,
        C: 'a;

    fn set_value<'a>(&'a self, name: &'a str, value: V) -> Self::Set<'a>;
    fn get_value<'a>(&'a self, name: &'a str) -> Self::Get<'a>;
    fn call<'a>(&'a self, ctx: &'a mut C, name: &'a str, args: Arguments<V>) -> Self::Call<'a>;
}
