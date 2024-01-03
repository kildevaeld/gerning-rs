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

pub trait Callable<C, V: Value> {
    fn signature(&self) -> Signature<V>;

    fn call(&self, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>>;
}

// #[cfg(feature = "service")]
// pub trait MethodCallable<S, C, V: Value> {
//     fn signature(&self) -> Signature<V>;
//     fn call(&self, this: &mut S, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>>;
// }

// impl<S, C, V: Value> MethodCallable<S, C, V> for Box<dyn MethodCallable<S, C, V>> {
//     fn signature(&self) -> Signature<V> {
//         (**self).signature()
//     }

//     fn call(&self, this: &mut S, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>> {
//         (**self).call(this, ctx, args)
//     }
// }

// #[cfg(feature = "service")]
// impl<'a, T, S, C, V: Value> MethodCallable<S, C, V> for &'a T
// where
//     T: MethodCallable<S, C, V>,
// {
//     fn signature(&self) -> Signature<V> {
//         (*self).signature()
//     }

//     fn call(&self, this: &mut S, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>> {
//         (*self).call(this, ctx, args)
//     }
// }

impl<F, C, U, E, V: Value> Callable<C, V> for F
where
    F: Fn(&mut C, Arguments<V>) -> Result<U, E>,
    E: Into<Error<V>>,
    U: Into<V> + Typed<V>,
{
    fn signature(&self) -> Signature<V> {
        Signature::new(Parameters::new(), U::get_type())
    }

    fn call(&self, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>> {
        (self)(ctx, args).map(|m| m.into()).map_err(|e| e.into())
    }
}

// impl<F, S, C, U, E, V: Value> MethodCallable<S, C, V> for F
// where
//     F: Fn(&mut S, &mut C, Arguments<V>) -> Result<U, E>,
//     E: Into<Error<V>>,
//     U: Into<V> + Typed<V>,
// {
//     fn signature(&self) -> Signature<V> {
//         Signature::new(Parameters::new(), U::get_type())
//     }

//     fn call(&self, this: &mut S, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>> {
//         (self)(this, ctx, args)
//             .map(|m| m.into())
//             .map_err(|e| e.into())
//     }
// }

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

pub trait CallableExt<C, V: Value>: Callable<C, V> {
    #[cfg(feature = "async")]
    fn into_async<E>(self) -> IntoAsync<Self, C, E, V>
    where
        Self: Sized,
        E: Executor,
    {
        IntoAsync {
            callable: self,
            _executor: PhantomData,
        }
    }

    fn boxed(self) -> Box<dyn Callable<C, V>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<C, T, V: Value> CallableExt<T, V> for C where C: Callable<T, V> {}

#[cfg(feature = "async")]
pub struct IntoAsync<C, T, E, V> {
    callable: C,
    _executor: PhantomData<(T, E, V)>,
}

#[cfg(feature = "async")]
impl<C, T, E, V> AsyncCallable<T, V> for IntoAsync<C, T, E, V>
where
    C: Callable<T, V> + Clone + Send + 'static,
    E: Executor + 'static,
    E::Error: core::fmt::Debug + Send + Sync + 'static,
    V: 'static + Value + Send,
    V::Type: Send,
    T: Send + Sync + 'static + Clone,
{
    type Future<'a> = Pin<Box<dyn Future<Output = Result<V, Error<V>>> + Send + 'a>>;
    fn signature(&self) -> Signature<V> {
        self.callable.signature()
    }

    fn call_async(&self, ctx: &mut T, args: Arguments<V>) -> Self::Future<'_> {
        let callable = self.callable.clone();
        let mut ctx = ctx.clone();
        Box::pin(async move {
            E::spawn_blocking(move || callable.call(&mut ctx, args))
                .await
                .map_err(|err| Error::Runtime(Box::new(err)))?
        })
    }
}
