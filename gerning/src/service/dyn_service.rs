use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
};

#[cfg(feature = "async")]
use futures_core::{ready, Future};
use hashbrown::HashMap;
use locket::LockApiWriteGuard;
#[cfg(feature = "async")]
use pin_project_lite::pin_project;

use super::{
    method::MethodCallable,
    service::Service,
    state::{HasState, StateType, SyncState},
    State,
};
#[cfg(feature = "async")]
use super::{
    method::{AsyncMethodCallable, BoxAsyncMethodCallable},
    service::AsyncService,
    state::AsyncStateType,
    LocalBoxAsyncMethodCallable,
};

use crate::{arguments::Arguments, signature::Signature, Error, Value};
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

// impl<T, S, C, V: Value> DynService<T, S, C, V>
// where
//     S: ServiceType,
//     T: HasState,
// {
//     fn method(&self, name: &str) -> Option<&S::Callable<T::State, C, V>> {
//         self.methods.get(name)
//     }
// }

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
    T::State: State<V>,
    V: Value,
    for<'a> T: 'a,
    for<'a> V: 'a,
    for<'a> C: 'a,
{
    type Get<'a> = GetFuture<'a, T, V>;
    type Set<'a> = SetFuture<'a, T, V>;
    type Call<'a> = AsyncMethodCallFuture<'a, S, T, C, V>;

    fn signature(&self) -> super::ServiceSignature<V> {
        let mut map: HashMap<String, Signature<V>> = HashMap::default();

        for (name, call) in &self.methods {
            map.insert(name.clone(), call.signature());
        }

        map.into()
    }

    fn set_value<'a>(&'a self, name: &'a str, value: V) -> Self::Set<'a> {
        let future = self.state.get();
        SetFuture {
            future,
            name,
            value: Some(value),
        }
    }

    fn get_value<'a>(&'a self, name: &'a str) -> Self::Get<'a> {
        let future = self.state.get();
        GetFuture { future, name }
    }

    fn call<'a>(&'a self, ctx: &'a mut C, name: &'a str, args: Arguments<V>) -> Self::Call<'a> {
        AsyncMethodCallFuture {
            state: AsyncMethodCallFutureState::Init {
                methods: &self.methods,
                state: self.state.get(),
                name,
                ctx: Some(ctx),
                args: Some(args),
            },
        }
    }
}

impl<T, S, C, V> Service<C, V> for DynService<T, S, C, V>
where
    S: ServiceType + 'static,
    S::Callable<T::State, C, V>: MethodCallable<T::State, C, V>,
    T: StateType<V>,
    T::State: State<V>,
    V: Value,
{
    fn signature(&self) -> super::ServiceSignature<V> {
        let mut map: HashMap<String, Signature<V>> = HashMap::default();

        for (name, call) in &self.methods {
            map.insert(name.clone(), call.signature());
        }

        map.into()
    }

    fn set_value(&self, name: &str, value: V) -> Result<(), Error<V>> {
        let mut lock = self.state.get()?;
        lock.get_mut().set(name, value)
    }

    fn get_value(&self, name: &str) -> Result<Option<V>, Error<V>> {
        let mut lock = self.state.get()?;
        lock.get_mut().get(name)
    }

    fn call(&self, ctx: &mut C, name: &str, args: Arguments<V>) -> Result<V, Error<V>> {
        let Some(method) = self.methods.get(name) else {
            return Err(Error::MethodNotFound);
        };

        let mut lock = self.state.get()?;
        method.call(lock.get_mut(), ctx, args)
    }
}

#[cfg(feature = "async")]
pin_project! {
    #[project = Proj]
    pub enum AsyncMethodCallFutureState<'a, S: ServiceType, T: AsyncStateType<V>, C, V: Value>
    where
        V: 'static,
        S::Callable<T::State, C, V>: AsyncMethodCallable<T::State, C, V>,
        T::State: 'a,
        T: 'a
    {
        Init {
            methods: &'a BTreeMap<String, S::Callable<T::State, C, V>>,
            #[pin]
            state: T::Future<'a>,
            ctx: Option<&'a mut C>,
            name: &'a str,
            args: Option<Arguments<V>>
        },
        Call {
            state: T::Ref<'a>,
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

#[cfg(feature = "async")]
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
                    let mut state = match ready!(state.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return core::task::Poll::Ready(Err(err.into())),
                    };

                    let Some(method) = methods.get(*name) else {
                        this.state.set(AsyncMethodCallFutureState::Done);
                        return core::task::Poll::Ready(Err(Error::MethodNotFound));
                    };

                    let ctx = ctx.take().expect("ctx");
                    let args = args.take().expect("args");
                    // let state = state.take().expect("state");

                    let unsafe_state = &mut state as *mut <T as AsyncStateType<V>>::Ref<'a>;

                    let future =
                        method.call_async(unsafe { &mut *unsafe_state }.get_mut(), ctx, args);

                    this.state
                        .set(AsyncMethodCallFutureState::Call { state, future })
                }
                Proj::Call { future, .. } => {
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

#[cfg(feature = "async")]
pin_project! {
    pub struct GetFuture<'a, S: 'a,V> where S: AsyncStateType<V>, V: Value  {
        #[pin]
        future: S::Future<'a>,
        name: &'a str,
    }
}

#[cfg(feature = "async")]
impl<'a, S, V> Future for GetFuture<'a, S, V>
where
    S: AsyncStateType<V> + 'a,
    S::State: State<V>,
    V: Value,
{
    type Output = Result<Option<V>, Error<V>>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match ready!(this.future.poll(cx)) {
            Ok(mut ret) => core::task::Poll::Ready(ret.get_mut().get(&this.name)),
            Err(err) => core::task::Poll::Ready(Err(err.into())),
        }
    }
}

#[cfg(feature = "async")]
pin_project! {
    pub struct SetFuture<'a, S: 'a, V> where S: AsyncStateType<V>, V: Value {
        #[pin]
        future: S::Future<'a>,
        name: &'a str,
        value: Option<V>
    }
}

#[cfg(feature = "async")]
impl<'a, S, V> Future for SetFuture<'a, S, V>
where
    S: AsyncStateType<V>,
    S::State: State<V>,
    V: Value,
{
    type Output = Result<(), Error<V>>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match ready!(this.future.poll(cx)) {
            Ok(mut ret) => core::task::Poll::Ready(
                ret.get_mut()
                    .set(&this.name, this.value.take().expect("value")),
            ),
            Err(err) => core::task::Poll::Ready(Err(err.into())),
        }
    }
}
