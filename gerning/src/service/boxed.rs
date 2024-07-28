#[cfg(feature = "async")]
mod r#async {
    use alloc::boxed::Box;
    use futures_core::future::BoxFuture;

    use crate::{arguments::Arguments, service::ServiceSignature, signature, Error, Value};

    use super::super::AsyncService;

    pub trait DynamicAsyncService<C, V: Value> {
        fn signature(&self) -> ServiceSignature<V>;
        fn set_value<'a>(&'a self, name: &'a str, value: V) -> BoxFuture<'a, Result<(), Error<V>>>;
        fn get_value<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Result<Option<V>, Error<V>>>;
        fn call<'a>(
            &'a self,
            ctx: &'a mut C,
            name: &'a str,
            args: Arguments<V>,
        ) -> BoxFuture<'a, Result<V, Error<V>>>;
    }

    impl<C, V: Value> DynamicAsyncService<C, V> for Box<dyn DynamicAsyncService<C, V> + Send + Sync> {
        fn signature(&self) -> ServiceSignature<V> {
            (**self).signature()
        }

        fn set_value<'a>(&'a self, name: &'a str, value: V) -> BoxFuture<'a, Result<(), Error<V>>> {
            (**self).set_value(name, value)
        }

        fn get_value<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Result<Option<V>, Error<V>>> {
            (**self).get_value(name)
        }

        fn call<'a>(
            &'a self,
            ctx: &'a mut C,
            name: &'a str,
            args: Arguments<V>,
        ) -> BoxFuture<'a, Result<V, Error<V>>> {
            (**self).call(ctx, name, args)
        }
    }

    pub struct BoxedDynamicAsyncService<T>(T);

    impl<T, C: 'static, V> DynamicAsyncService<C, V> for BoxedDynamicAsyncService<T>
    where
        V: Value + 'static,
        T: AsyncService<C, V>,
        for<'a> T::Set<'a>: Send,
        for<'a> T::Get<'a>: Send,
        for<'a> T::Call<'a>: Send,
    {
        fn signature(&self) -> ServiceSignature<V> {
            self.0.signature()
        }
        fn set_value<'a>(&'a self, name: &'a str, value: V) -> BoxFuture<'a, Result<(), Error<V>>> {
            Box::pin(self.0.set_value(name, value))
        }

        fn get_value<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Result<Option<V>, Error<V>>> {
            Box::pin(self.0.get_value(name))
        }

        fn call<'a>(
            &'a self,
            ctx: &'a mut C,
            name: &'a str,
            args: Arguments<V>,
        ) -> BoxFuture<'a, Result<V, Error<V>>> {
            Box::pin(self.0.call(ctx, name, args))
        }
    }

    impl<C: 'static, V: Value + 'static> AsyncService<C, V> for BoxAsyncService<C, V> {
        type Get<'a> = BoxFuture<'a, Result<Option<V>, Error<V>>>;

        type Set<'a> = BoxFuture<'a, Result<(), Error<V>>>;

        type Call<'a> = BoxFuture<'a, Result<V, Error<V>>>;

        fn signature(&self) -> crate::service::ServiceSignature<V> {
            <Self as DynamicAsyncService<C, V>>::signature(self)
        }

        fn set_value<'a>(&'a self, name: &'a str, value: V) -> Self::Set<'a> {
            <Self as DynamicAsyncService<C, V>>::set_value(self, name, value)
        }

        fn get_value<'a>(&'a self, name: &'a str) -> Self::Get<'a> {
            <Self as DynamicAsyncService<C, V>>::get_value(self, name)
        }

        fn call<'a>(&'a self, ctx: &'a mut C, name: &'a str, args: Arguments<V>) -> Self::Call<'a> {
            <Self as DynamicAsyncService<C, V>>::call(self, ctx, name, args)
        }
    }

    pub fn box_service<C, V: Value, T>(service: T) -> BoxAsyncService<C, V>
    where
        C: 'static,
        V: Value + 'static,
        T: AsyncService<C, V> + Send + Sync + 'static,
        for<'a> T::Set<'a>: Send,
        for<'a> T::Get<'a>: Send,
        for<'a> T::Call<'a>: Send,
    {
        Box::new(BoxedDynamicAsyncService(service))
    }

    pub type BoxAsyncService<C, V> = Box<dyn DynamicAsyncService<C, V> + Send + Sync>;
}

#[cfg(feature = "async")]
pub use r#async::{box_service, BoxAsyncService};
