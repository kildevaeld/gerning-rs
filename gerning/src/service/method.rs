use alloc::boxed::Box;

use crate::{
    arguments::Arguments,
    signature::{Parameters, Signature},
    Error, Typed, Value,
};

pub trait MethodCallable<S, C, V: Value> {
    fn signature(&self) -> Signature<V>;
    fn call(&self, this: &mut S, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>>;
}

impl<S, C, V: Value> MethodCallable<S, C, V> for Box<dyn MethodCallable<S, C, V>> {
    fn signature(&self) -> Signature<V> {
        (**self).signature()
    }

    fn call(&self, this: &mut S, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>> {
        (**self).call(this, ctx, args)
    }
}

impl<F, S, C, U, E, V: Value> MethodCallable<S, C, V> for F
where
    F: Fn(&mut S, &mut C, Arguments<V>) -> Result<U, E>,
    E: Into<Error<V>>,
    U: Into<V> + Typed<V>,
{
    fn signature(&self) -> Signature<V> {
        Signature::new(Parameters::new(), U::get_type())
    }

    fn call(&self, this: &mut S, ctx: &mut C, args: Arguments<V>) -> Result<V, Error<V>> {
        (self)(this, ctx, args)
            .map(|m| m.into())
            .map_err(|e| e.into())
    }
}

#[cfg(feature = "async")]
mod service_impl {
    use crate::{signature::Parameters, Resultable, Typed};

    use super::*;
    use core::{future::IntoFuture, pin::Pin};
    use futures_core::{
        future::{BoxFuture, LocalBoxFuture},
        Future,
    };
    pub trait AsyncMethodCallable<S, C, V: Value> {
        type Future<'a>: Future<Output = Result<V, Error<V>>>
        where
            Self: 'a,
            C: 'a,
            S: 'a;
        fn signature(&self) -> Signature<V>;

        fn call_async<'a>(
            &'a self,
            this: &'a mut S,
            ctx: &'a mut C,
            args: Arguments<V>,
        ) -> Self::Future<'a>;
    }

    pub trait AsyncMethodCallableExt<S, C, V: Value>: AsyncMethodCallable<S, C, V> {
        fn boxed(self) -> BoxAsyncMethodCallable<'static, S, C, V>
        where
            Self: Sized + 'static + Send + Sync,
            for<'a> Self::Future<'a>: Send,
            V: 'static,
            for<'a> C: 'a,
        {
            Box::new(self)
        }

        fn boxed_local(self) -> LocalBoxAsyncMethodCallable<'static, S, C, V>
        where
            Self: Sized + 'static + Send + Sync,
            V: 'static,
            C: 'static,
        {
            Box::new(self)
        }
    }

    impl<T, S, C, V: Value> AsyncMethodCallableExt<S, C, V> for T where T: AsyncMethodCallable<S, C, V> {}

    pub type BoxAsyncMethodCallable<'a, S, C, V> =
        Box<dyn internal::BoxAsyncCall<S, C, V> + Send + Sync + 'a>;

    pub type LocalBoxAsyncMethodCallable<'a, S, C, V> =
        Box<dyn internal::BoxLocalAsyncCall<S, C, V> + Send + Sync + 'a>;

    mod internal {
        use futures_core::future::LocalBoxFuture;

        use super::*;

        pub trait BoxAsyncCall<S, C, V: Value> {
            fn signature(&self) -> Signature<V>;
            fn call<'a>(
                &'a self,
                this: &'a mut S,
                ctx: &'a mut C,
                args: super::Arguments<V>,
            ) -> BoxFuture<'a, Result<V, Error<V>>>;
        }

        impl<T, S, C, V> BoxAsyncCall<S, C, V> for T
        where
            T: AsyncMethodCallable<S, C, V>,
            for<'a> T::Future<'a>: Send,
            V: Value + 'static,
            C: 'static,
        {
            fn signature(&self) -> Signature<V> {
                <T as AsyncMethodCallable<S, C, V>>::signature(self)
            }

            fn call<'a>(
                &'a self,
                this: &'a mut S,
                ctx: &'a mut C,
                args: super::Arguments<V>,
            ) -> BoxFuture<'a, Result<V, Error<V>>> {
                Box::pin(<T as AsyncMethodCallable<S, C, V>>::call_async(
                    self, this, ctx, args,
                ))
            }
        }

        pub trait BoxLocalAsyncCall<S, C, V: Value> {
            fn signature(&self) -> Signature<V>;
            fn call<'a>(
                &'a self,
                this: &'a mut S,
                ctx: &'a mut C,
                args: super::Arguments<V>,
            ) -> LocalBoxFuture<'a, Result<V, Error<V>>>;
        }

        impl<T, S, C, V> BoxLocalAsyncCall<S, C, V> for T
        where
            T: AsyncMethodCallable<S, C, V>,
            V: Value + 'static,
            C: 'static,
        {
            fn signature(&self) -> Signature<V> {
                <T as AsyncMethodCallable<S, C, V>>::signature(self)
            }

            fn call<'a>(
                &'a self,
                this: &'a mut S,
                ctx: &'a mut C,
                args: super::Arguments<V>,
            ) -> LocalBoxFuture<'a, Result<V, Error<V>>> {
                Box::pin(<T as AsyncMethodCallable<S, C, V>>::call_async(
                    self, this, ctx, args,
                ))
            }
        }
    }

    impl<S, C, V: Value + 'static> AsyncMethodCallable<S, C, V>
        for BoxAsyncMethodCallable<'static, S, C, V>
    {
        type Future<'a> = BoxFuture<'a, Result<V, Error<V>>> where C:'a, S: 'a;
        fn signature(&self) -> Signature<V> {
            (**self).signature()
        }
        fn call_async<'a>(
            &'a self,
            this: &'a mut S,
            ctx: &'a mut C,
            args: Arguments<V>,
        ) -> Self::Future<'a> {
            (**self).call(this, ctx, args)
        }
    }

    impl<S, C, V: Value + 'static> AsyncMethodCallable<S, C, V>
        for LocalBoxAsyncMethodCallable<'static, S, C, V>
    {
        type Future<'a> = LocalBoxFuture<'a, Result<V, Error<V>>> where C:'a, S: 'a;
        fn signature(&self) -> Signature<V> {
            (**self).signature()
        }
        fn call_async<'a>(
            &'a self,
            this: &'a mut S,
            ctx: &'a mut C,
            args: Arguments<V>,
        ) -> Self::Future<'a> {
            (**self).call(this, ctx, args)
        }
    }

    impl<F, S, U, C, V: Value> AsyncMethodCallable<S, C, V> for F
    where
        F: Fn(&mut S, &mut C, Arguments<V>) -> U + Clone,
        for<'a> F: 'a,
        for<'a> U: IntoFuture + 'a,
        for<'a> C: 'a,
        for<'a> S: 'a,
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

        fn call_async<'a>(
            &'a self,
            this: &'a mut S,
            ctx: &'a mut C,
            args: Arguments<V>,
        ) -> Self::Future<'a> {
            let future = (self)(this, ctx, args);
            let future = async move {
                let ret = future.into_future().await;
                ret.into_result().map(Into::into).map_err(Into::into)
            };

            Box::pin(future)
        }
    }
}

#[cfg(feature = "async")]
pub use service_impl::*;
