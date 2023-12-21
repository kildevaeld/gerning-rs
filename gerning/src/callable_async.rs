use crate::signature::{Parameters, Signature};
use crate::traits::{Typed, Value};
use crate::{arguments::Arguments, Error, Resultable};
use alloc::boxed::Box;
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use futures_core::future::{BoxFuture, LocalBoxFuture};

pub trait AsyncCallable<C, V: Value> {
    type Future<'a>: Future<Output = Result<V, Error<V>>>
    where
        Self: 'a,
        C: 'a;
    fn signature(&self) -> Signature<V>;

    fn call_async<'a>(&'a self, ctx: &'a mut C, args: Arguments<V>) -> Self::Future<'a>;
}

pub trait AsyncCallableExt<C, V: Value>: AsyncCallable<C, V> {
    fn boxed(self) -> BoxAsyncCallable<C, V>
    where
        Self: Sized + 'static + Send + Sync,
        for<'a> Self::Future<'a>: Send,
        V: 'static,
        for<'a> C: 'a,
    {
        Box::new(self)
    }

    fn boxed_local(self) -> LocalBoxAsyncCallable<C, V>
    where
        Self: Sized + 'static + Send + Sync,
        V: 'static,
        C: 'static,
    {
        Box::new(self)
    }
}

impl<T, C, V: Value> AsyncCallableExt<C, V> for T where T: AsyncCallable<C, V> {}

pub type BoxAsyncCallable<C, V> = Box<dyn internal::BoxAsyncCall<C, V> + Send + Sync>;

pub type LocalBoxAsyncCallable<C, V> = Box<dyn internal::BoxLocalAsyncCall<C, V> + Send + Sync>;

mod internal {
    use futures_core::future::LocalBoxFuture;

    use super::*;

    pub trait BoxAsyncCall<C, V: Value> {
        fn signature(&self) -> Signature<V>;
        fn call<'a>(
            &'a self,
            ctx: &'a mut C,
            args: super::Arguments<V>,
        ) -> BoxFuture<'a, Result<V, Error<V>>>;
    }

    impl<T, C, V> BoxAsyncCall<C, V> for T
    where
        T: AsyncCallable<C, V>,
        for<'a> T::Future<'a>: Send,
        V: Value + 'static,
        C: 'static,
    {
        fn signature(&self) -> Signature<V> {
            <T as AsyncCallable<C, V>>::signature(self)
        }

        fn call<'a>(
            &'a self,
            ctx: &'a mut C,
            args: super::Arguments<V>,
        ) -> BoxFuture<'a, Result<V, Error<V>>> {
            Box::pin(<T as AsyncCallable<C, V>>::call_async(self, ctx, args))
        }
    }

    pub trait BoxLocalAsyncCall<C, V: Value> {
        fn signature(&self) -> Signature<V>;
        fn call<'a>(
            &'a self,
            ctx: &'a mut C,
            args: super::Arguments<V>,
        ) -> LocalBoxFuture<'a, Result<V, Error<V>>>;
    }

    impl<T, C, V> BoxLocalAsyncCall<C, V> for T
    where
        T: AsyncCallable<C, V>,
        V: Value + 'static,
        C: 'static,
    {
        fn signature(&self) -> Signature<V> {
            <T as AsyncCallable<C, V>>::signature(self)
        }

        fn call<'a>(
            &'a self,
            ctx: &'a mut C,
            args: super::Arguments<V>,
        ) -> LocalBoxFuture<'a, Result<V, Error<V>>> {
            Box::pin(<T as AsyncCallable<C, V>>::call_async(self, ctx, args))
        }
    }
}

impl<C: 'static, V: Value + 'static> AsyncCallable<C, V> for BoxAsyncCallable<C, V> {
    type Future<'a> = BoxFuture<'a, Result<V, Error<V>>>;
    fn signature(&self) -> Signature<V> {
        (**self).signature()
    }
    fn call_async<'a>(&'a self, ctx: &'a mut C, args: Arguments<V>) -> Self::Future<'a> {
        (**self).call(ctx, args)
    }
}

impl<C: 'static, V: Value + 'static> AsyncCallable<C, V> for LocalBoxAsyncCallable<C, V> {
    type Future<'a> = LocalBoxFuture<'a, Result<V, Error<V>>>;
    fn signature(&self) -> Signature<V> {
        (**self).signature()
    }
    fn call_async<'a>(&'a self, ctx: &'a mut C, args: Arguments<V>) -> Self::Future<'a> {
        (**self).call(ctx, args)
    }
}

impl<F, U, C, V: Value> AsyncCallable<C, V> for F
where
    F: Fn(&mut C, Arguments<V>) -> U + Clone,
    for<'a> F: 'a,
    for<'a> U: IntoFuture + 'a,
    for<'a> C: 'a,
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

    fn call_async<'a>(&'a self, ctx: &'a mut C, args: Arguments<V>) -> Self::Future<'a> {
        let future = (self)(ctx, args);
        let future = async move {
            let ret = future.into_future().await;
            ret.into_result().map(Into::into).map_err(Into::into)
        };

        Box::pin(future)
    }
}
