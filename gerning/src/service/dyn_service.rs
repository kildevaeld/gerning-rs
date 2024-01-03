use core::cell::RefCell;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    sync::Arc,
};
use avagarden::sync::Mutex;
use futures_core::Future;
use pin_project_lite::pin_project;

use super::{
    method::MethodCallable,
    service::Service,
    state::{HasState, StateType, SyncState},
    LocalBoxAsyncMethodCallable,
};
#[cfg(feature = "async")]
use super::{
    method::{AsyncMethodCallable, BoxAsyncMethodCallable},
    service::AsyncService,
    state::AsyncStateType,
};

use crate::{arguments::Arguments, Error, Value};
pub trait ServiceType {
    type Callable<S, C, V>;
    type State<T>;
}

pub struct Sync;

impl ServiceType for Sync {
    type Callable<S, C, V> = Box<dyn MethodCallable<S, C, V>>;
    type State<T> = SyncState<T>;
}

#[cfg(feature = "async")]
pub struct Async;

#[cfg(feature = "async")]
impl ServiceType for Async {
    type Callable<S, C, V> = LocalBoxAsyncMethodCallable<'static, S, C, V>;
    type State<T> = SyncState<T>;
}

#[cfg(feature = "async")]
pub struct SendAsync;

#[cfg(feature = "async")]
impl ServiceType for SendAsync {
    type Callable<S, C, V> = BoxAsyncMethodCallable<'static, S, C, V>;
    type State<T> = SyncState<T>;
}

pub struct DynService<T: HasState, S: ServiceType, C, V: Value> {
    state: T,
    methods: BTreeMap<String, S::Callable<T::State, C, V>>,
}

impl<T, S, C, V: Value> DynService<T, S, C, V>
where
    S: ServiceType,
    T: HasState,
{
    fn method(&self, name: &str) -> Option<&S::Callable<T::State, C, V>> {
        self.methods.get(name)
    }
}

impl<T, C, V: Value> DynService<T, Sync, C, V>
where
    T: HasState,
{
    pub fn new(state: T) -> DynService<T, Sync, C, V> {
        DynService {
            state,
            methods: Default::default(),
        }
    }
}

#[cfg(feature = "async")]
impl<T, C, V: Value> DynService<T, Async, C, V>
where
    T: HasState,
{
    pub fn new_async(state: T) -> DynService<T, Async, C, V> {
        DynService {
            state,
            methods: Default::default(),
        }
    }

    pub fn new_async_send(state: T) -> DynService<T, SendAsync, C, V> {
        DynService {
            state,
            methods: Default::default(),
        }
    }
}

impl<T, C, V> DynService<T, Sync, C, V>
where
    T: HasState,
    V: Value,
{
    pub fn register<U>(&mut self, name: &str, method: U) -> &mut Self
    where
        U: MethodCallable<T::State, C, V> + 'static,
    {
        self.methods.insert(name.to_string(), Box::new(method));
        self
    }
}

#[cfg(feature = "async")]
impl<T, C, V> DynService<T, Async, C, V>
where
    T: HasState,
    V: Value + 'static,
{
    pub fn register<U>(&mut self, name: &str, method: U) -> &mut Self
    where
        U: AsyncMethodCallable<T::State, C, V> + 'static + Send + core::marker::Sync,
        for<'a> C: 'a,
    {
        self.methods.insert(name.to_string(), Box::new(method));
        self
    }
}

#[cfg(feature = "async")]
impl<T, C, V> DynService<T, SendAsync, C, V>
where
    T: HasState,
    V: Value + 'static,
{
    pub fn register<U>(&mut self, name: &str, method: U) -> &mut Self
    where
        U: AsyncMethodCallable<T::State, C, V> + 'static + Send + core::marker::Sync,
        for<'a> U::Future<'a>: Send,
        for<'a> C: 'a,
    {
        self.methods.insert(name.to_string(), Box::new(method));
        self
    }
}

#[cfg(feature = "async")]
impl<S, T, C, V> AsyncService<C, V> for DynService<T, S, C, V>
where
    S: ServiceType + 'static,
    S::Callable<T::State, C, V>: AsyncMethodCallable<T::State, C, V>,
    T: AsyncStateType<V>,
    V: Value,
    for<'a> T: 'a,
    for<'a> V: 'a,
    for<'a> C: 'a,
{
    type Get<'a> = T::Get<'a>;
    type Set<'a> = T::Set<'a>;
    type Call<'a> = T::Call<'a, AsyncMethodCallFuture<'a, S, T, C, V>>;

    fn set_value<'a>(&'a self, name: &'a str, value: V) -> Self::Set<'a> {
        self.state.set(name, value)
    }

    fn get_value<'a>(&'a self, name: &'a str) -> Self::Get<'a> {
        self.state.get(name)
    }

    fn call<'a>(&'a self, ctx: &'a mut C, name: &'a str, args: Arguments<V>) -> Self::Call<'a> {
        self.state.call(|state| AsyncMethodCallFuture {
            state: AsyncMethodCallFutureState::Init {
                methods: &self.methods,
                state: Some(state),
                name,
                ctx: Some(ctx),
                args: Some(args),
            },
        })
    }
}

impl<T, S, C, V> Service<C, V> for DynService<T, S, C, V>
where
    S: ServiceType + 'static,
    S::Callable<T::State, C, V>: MethodCallable<T::State, C, V>,
    T: StateType<V>,
    V: Value,
{
    fn set_value(&self, name: &str, value: V) -> Result<(), Error<V>> {
        self.state.set(name, value)
    }

    fn get_value(&self, name: &str) -> Result<Option<V>, Error<V>> {
        self.state.get(name)
    }

    fn call(&self, ctx: &mut C, name: &str, args: Arguments<V>) -> Result<V, Error<V>> {
        let Some(method) = self.methods.get(name) else {
            return Err(Error::MethodNotFound);
        };
        self.state.call(|state| method.call(state, ctx, args))
    }
}

#[cfg(feature = "async")]
pin_project! {
    #[project = Proj]
    pub enum AsyncMethodCallFutureState<'a, S: ServiceType, T: AsyncStateType<V>, C, V: Value>
    where
        V: 'static,
        S::Callable<T::State, C, V>: AsyncMethodCallable<T::State, C, V>
    {
        Init {
            methods: &'a BTreeMap<String, S::Callable<T::State, C, V>>,
            state: Option<&'a mut T::State>,
            ctx: Option<&'a mut C>,
            name: &'a str,
            args: Option<Arguments<V>>
        },
        Call {
            #[pin]
            future: <S::Callable<T::State, C, V> as AsyncMethodCallable<T::State, C, V>>::Future<'a>,
        },
        Done,
    }
}

#[cfg(feature = "async")]
pin_project! {
    pub struct AsyncMethodCallFuture<'a, S: ServiceType, T: AsyncStateType<V>, C, V: Value>
    where
        V: 'static,
        S::Callable<T::State, C, V>: AsyncMethodCallable<T::State, C, V>
    {
        #[pin]
        state: AsyncMethodCallFutureState<'a, S, T, C, V>
    }
}

impl<'a, S: ServiceType, T: AsyncStateType<V>, C, V: Value> core::future::Future
    for AsyncMethodCallFuture<'a, S, T, C, V>
where
    S::Callable<T::State, C, V>: AsyncMethodCallable<T::State, C, V>,
{
    type Output = Result<V, Error<V>>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();
            match this.state.as_mut().project() {
                Proj::Init {
                    methods,
                    state,
                    ctx,
                    name,
                    args,
                } => {
                    let Some(method) = methods.get(*name) else {
                        this.state.set(AsyncMethodCallFutureState::Done);
                        return core::task::Poll::Ready(Err(Error::MethodNotFound));
                    };

                    let ctx = ctx.take().expect("ctx");
                    let args = args.take().expect("args");
                    let state = state.take().expect("state");

                    let future = method.call_async(state, ctx, args);

                    this.state.set(AsyncMethodCallFutureState::Call { future })
                }
                Proj::Call { future } => {
                    let ret = futures_core::ready!(future.poll(cx));

                    this.state.set(AsyncMethodCallFutureState::Done);

                    return core::task::Poll::Ready(ret);
                }
                Proj::Done => {
                    panic!("poll after done")
                }
            }
        }
    }
}
