use crate::signature::{Parameters, Signature};
use crate::traits::{Typed, Value};
use crate::{arguments::Arguments, Error, Resultable};
use alloc::boxed::Box;
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use futures_core::future::{BoxFuture, LocalBoxFuture};

pub trait AsyncCallable<V: Value> {
    type Future<'a>: Future<Output = Result<V, Error<V>>>
    where
        Self: 'a;
    fn signature(&self) -> Signature<V>;

    fn call_async(&self, args: Arguments<V>) -> Self::Future<'_>;
}

pub trait AsyncCallableExt<V: Value>: AsyncCallable<V> {
    fn boxed(self) -> BoxAsyncCallable<V>
    where
        Self: Sized + 'static + Send + Sync,
        for<'a> Self::Future<'a>: Send,
        V: 'static,
    {
        Box::new(self)
    }

    fn boxed_local(self) -> LocalBoxAsyncCallable<V>
    where
        Self: Sized + 'static + Send + Sync,
        V: 'static,
    {
        Box::new(self)
    }
}

impl<T, V: Value> AsyncCallableExt<V> for T where T: AsyncCallable<V> {}

pub type BoxAsyncCallable<V> = Box<dyn internal::BoxAsyncCall<V> + Send + Sync>;

pub type LocalBoxAsyncCallable<V> = Box<dyn internal::BoxLocalAsyncCall<V> + Send + Sync>;

mod internal {
    use futures_core::future::LocalBoxFuture;

    use super::*;

    pub trait BoxAsyncCall<V: Value> {
        fn signature(&self) -> Signature<V>;
        fn call(&self, args: super::Arguments<V>) -> BoxFuture<'_, Result<V, Error<V>>>;
    }

    impl<T, V> BoxAsyncCall<V> for T
    where
        T: AsyncCallable<V>,
        for<'a> T::Future<'a>: Send,
        V: Value + 'static,
    {
        fn signature(&self) -> Signature<V> {
            <T as AsyncCallable<V>>::signature(self)
        }

        fn call(&self, args: super::Arguments<V>) -> BoxFuture<'_, Result<V, Error<V>>> {
            Box::pin(<T as AsyncCallable<V>>::call_async(self, args))
        }
    }

    pub trait BoxLocalAsyncCall<V: Value> {
        fn signature(&self) -> Signature<V>;
        fn call(&self, args: super::Arguments<V>)
            -> LocalBoxFuture<'_, Result<V, Error<V>>>;
    }

    impl<T, V> BoxLocalAsyncCall<V> for T
    where
        T: AsyncCallable<V>,
        V: Value + 'static,
    {
        fn signature(&self) -> Signature<V> {
            <T as AsyncCallable<V>>::signature(self)
        }

        fn call(
            &self,
            args: super::Arguments<V>,
        ) -> LocalBoxFuture<'_, Result<V, Error<V>>> {
            Box::pin(<T as AsyncCallable<V>>::call_async(self, args))
        }
    }
}

impl<V: Value + 'static> AsyncCallable<V> for BoxAsyncCallable<V> {
    type Future<'a> = BoxFuture<'a, Result<V, Error<V>>>;
    fn signature(&self) -> Signature<V> {
        (**self).signature()
    }
    fn call_async(&self, args: Arguments<V>) -> Self::Future<'_> {
        (**self).call(args)
    }
}

impl<V: Value + 'static> AsyncCallable<V> for LocalBoxAsyncCallable<V> {
    type Future<'a> = LocalBoxFuture<'a, Result<V, Error<V>>>;
    fn signature(&self) -> Signature<V> {
        (**self).signature()
    }
    fn call_async(&self, args: Arguments<V>) -> Self::Future<'_> {
        (**self).call(args)
    }
}

impl<F, U, V: Value> AsyncCallable<V> for F
where
    F: Fn(Arguments<V>) -> U + Clone,
    for<'a> F: 'a,
    for<'a> U: IntoFuture + 'a,
    U::Output: Resultable,
    <U::Output as Resultable>::Error: Into<Error<V>>,
    <U::Output as Resultable>::Ok: Into<V> + Typed<V>,
{
    type Future<'a> = Pin<Box<dyn Future<Output = Result<V, Error<V>>> + 'a>>;

    fn signature(&self) -> Signature<V> {
        Signature::new(
            Parameters::new(),
            <<U::Output as Resultable>::Ok as Typed<V>>::get_type(),
        )
    }

    fn call_async(&self, args: Arguments<V>) -> Self::Future<'_> {
        let future = (self)(args);
        let future = async move {
            let ret = future.into_future().await;
            ret.into_result().map(Into::into).map_err(Into::into)
        };

        Box::pin(future)
    }
}
