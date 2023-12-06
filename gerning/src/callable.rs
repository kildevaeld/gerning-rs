#[cfg(feature = "async")]
use crate::AsyncCallable;
use crate::{
    arguments::Arguments,
    error::Error,
    signature::{Parameters, Signature},
    traits::{Typed, Value},
};
use alloc::boxed::Box;
#[cfg(feature = "async")]
use core::{marker::PhantomData, pin::Pin};
#[cfg(feature = "async")]
use futures_core::Future;

pub trait Callable<V: Value> {
    fn signature(&self) -> Signature<V>;

    fn call(&self, args: Arguments<V>) -> Result<V, Error<V>>;
}

impl<F, U, E, V: Value> Callable<V> for F
where
    F: Fn(Arguments<V>) -> Result<U, E>,
    E: Into<Error<V>>,
    U: Into<V> + Typed<V>,
{
    fn signature(&self) -> Signature<V> {
        Signature::new(Parameters::new(), U::get_type())
    }

    fn call(&self, args: Arguments<V>) -> Result<V, Error<V>> {
        (self)(args).map(|m| m.into()).map_err(|e| e.into())
    }
}

#[cfg(feature = "async")]
pub trait Executor {
    type Error;
    fn spawn_blocking<F: FnOnce() -> R + 'static + Send, R: Send + 'static>(
        func: F,
    ) -> Pin<Box<dyn Future<Output = Result<R, Self::Error>> + Send>>;
}

#[cfg(feature = "tokio")]
pub struct Tokio;

#[cfg(feature = "tokio")]
impl Executor for Tokio {
    type Error = tokio::task::JoinError;
    fn spawn_blocking<F: FnOnce() -> R + 'static + Send, R: Send + 'static>(
        func: F,
    ) -> Pin<Box<dyn Future<Output = Result<R, Self::Error>> + Send>> {
        Box::pin(tokio::task::spawn_blocking(func))
    }
}

#[cfg(feature = "smol")]
pub struct Smol;

#[cfg(feature = "smol")]
impl Executor for Smol {
    type Error = ();
    fn spawn_blocking<F: FnOnce() -> R + 'static + Send, R: Send + 'static>(
        func: F,
    ) -> Pin<Box<dyn Future<Output = Result<R, Self::Error>> + Send>> {
        Box::pin(async move { Ok(smol::unblock(func).await) })
    }
}

pub trait CallableExt<V: Value>: Callable<V> {
    #[cfg(feature = "async")]
    fn into_async<E>(self) -> IntoAsync<Self, E, V>
    where
        Self: Sized,
        E: Executor,
    {
        IntoAsync {
            callable: self,
            _executor: PhantomData,
        }
    }

    fn boxed(self) -> Box<dyn Callable<V>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<C, V: Value> CallableExt<V> for C where C: Callable<V> {}

#[cfg(feature = "async")]
pub struct IntoAsync<C, E, V> {
    callable: C,
    _executor: PhantomData<(E, V)>,
}

#[cfg(feature = "async")]
impl<C, E, V> AsyncCallable<V> for IntoAsync<C, E, V>
where
    C: Callable<V> + Clone + Send + 'static,
    E: Executor + 'static,
    E::Error: core::fmt::Debug + Send + Sync + 'static,
    V: 'static + Value + Send,
    V::Type: Send,
{
    type Future<'a> = Pin<Box<dyn Future<Output = Result<V, Error<V>>> + Send + 'a>>;
    fn signature(&self) -> Signature<V> {
        self.callable.signature()
    }

    fn call_async(&self, args: Arguments<V>) -> Self::Future<'_> {
        let callable = self.callable.clone();
        Box::pin(async move {
            

            E::spawn_blocking(move || callable.call(args))
                .await
                .map_err(|err| Error::Runtime(Box::new(err)))?
        })
    }
}
