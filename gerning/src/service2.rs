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

use crate::{arguments::Arguments, AsyncMethodCallable, Error, MethodCallable, Value};

pub trait HasState {
    type State;
}

pub trait StateType<V: Value>: HasState {
    // type State;
    fn set(&self, name: &str, value: V) -> Result<(), Error<V>>;
    fn get(&self, name: &str) -> Result<Option<V>, Error<V>>;
    fn call<F>(&self, func: F) -> Result<V, Error<V>>
    where
        F: FnOnce(&mut Self::State) -> Result<V, Error<V>>;
}

#[cfg(feature = "async")]
pub trait AsyncStateType<V: Value>: HasState {
    type Get<'a>: Future<Output = Result<Option<V>, Error<V>>>
    where
        Self: 'a;
    type Set<'a>: Future<Output = Result<(), Error<V>>>
    where
        Self: 'a;
    type Call<'a, F>: Future<Output = Result<V, Error<V>>>
    where
        Self: 'a;

    fn set<'a>(&'a self, name: &'a str, value: V) -> Self::Set<'a>;
    fn get<'a>(&'a self, name: &'a str) -> Self::Get<'a>;
    fn call<'a, F, U>(&self, func: F) -> Self::Call<'a, U>
    where
        Self::State: 'a,
        F: FnOnce(&'a mut Self::State) -> U,
        U: Future<Output = Result<V, Error<V>>>;
}

pub struct SendState<T> {
    state: Arc<Mutex<T>>,
}

impl<T> HasState for SendState<T> {
    type State = T;
}

impl<T> SendState<T> {
    pub fn new(state: T) -> SendState<T> {
        SendState {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

#[cfg(feature = "async")]
impl<T: State<V>, V: Value> AsyncStateType<V> for SendState<T> {
    type Get<'a> = core::future::Ready<Result<Option<V>, Error<V>>> where T: 'a;
    type Set<'a> = core::future::Ready<Result<(), Error<V>>> where T: 'a;
    type Call<'a, U> = core::future::Ready<Result<V, Error<V>>> where T: 'a;

    fn set<'a>(&'a self, name: &'a str, value: V) -> Self::Set<'a> {
        core::future::ready(self.state.lock().set(name, value))
    }

    fn get<'a>(&'a self, name: &'a str) -> Self::Get<'a> {
        core::future::ready(self.state.lock().get(name))
    }

    fn call<'a, F, U>(&self, func: F) -> Self::Call<'a, U>
    where
        Self::State: 'a,
        F: FnOnce(&'a mut Self::State) -> U,
        U: Future<Output = Result<V, Error<V>>>,
    {
        let mut state = self.state.lock();
        // func(&mut *state)
        todo!()
    }
}

pub struct SyncState<T> {
    state: Rc<RefCell<T>>,
}

impl<T> SyncState<T> {
    pub fn new(state: T) -> SyncState<T> {
        SyncState {
            state: Rc::new(RefCell::new(state)),
        }
    }
}

impl<T> HasState for SyncState<T> {
    type State = T;
}

impl<T: State<V>, V: Value> StateType<V> for SyncState<T> {
    fn set(&self, name: &str, value: V) -> Result<(), Error<V>> {
        self.state.borrow_mut().set(name, value)
    }

    fn get(&self, name: &str) -> Result<Option<V>, Error<V>> {
        self.state.borrow().get(name)
    }

    fn call<F>(&self, func: F) -> Result<V, Error<V>>
    where
        F: FnOnce(&mut Self::State) -> Result<V, Error<V>>,
    {
        let mut state = self.state.borrow_mut();
        func(&mut *state)
    }
}

#[cfg(feature = "async")]
impl<T: State<V>, V: Value> AsyncStateType<V> for SyncState<T> {
    type Get<'a> = core::future::Ready<Result<Option<V>, Error<V>>> where T: 'a;
    type Set<'a> = core::future::Ready<Result<(), Error<V>>> where T: 'a;
    type Call<'a, U> = core::future::Ready<Result<V, Error<V>>> where T: 'a;

    fn set<'a>(&'a self, name: &'a str, value: V) -> Self::Set<'a> {
        core::future::ready(self.state.borrow_mut().set(name, value))
    }

    fn get<'a>(&'a self, name: &'a str) -> Self::Get<'a> {
        core::future::ready(self.state.borrow().get(name))
    }

    fn call<'a, F, U>(&self, func: F) -> Self::Call<'a, U>
    where
        Self::State: 'a,
        F: FnOnce(&'a mut Self::State) -> U,
        U: Future<Output = Result<V, Error<V>>>,
    {
        let mut state = self.state.borrow_mut();
        // func(&mut *state)
        todo!()
    }
}

pub trait State<V: Value> {
    fn set(&mut self, name: &str, value: V) -> Result<(), Error<V>>;
    fn get(&self, name: &str) -> Result<Option<V>, Error<V>>;
}

impl<V: Value + Clone> State<V> for BTreeMap<String, V> {
    fn get(&self, name: &str) -> Result<Option<V>, Error<V>> {
        Ok(self.get(name).cloned())
    }

    fn set(&mut self, name: &str, value: V) -> Result<(), Error<V>> {
        self.insert(name.to_string(), value);
        Ok(())
    }
}

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
    type Callable<S, C, V> = crate::BoxAsyncMethodCallable<'static, S, C, V>;
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
}

impl<T, C, V> DynService<T, Sync, C, V>
where
    T: HasState,
    V: Value,
{
    pub fn register<U>(&mut self, name: &str, method: U) -> &mut Self
    where
        U: crate::MethodCallable<T::State, C, V> + 'static,
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
        U: crate::AsyncMethodCallable<T::State, C, V> + 'static + Send + core::marker::Sync,
        for<'a> U::Future<'a>: Send,
        for<'a> C: 'a,
    {
        self.methods.insert(name.to_string(), Box::new(method));
        self
    }
}

#[cfg(feature = "async")]
impl<T, C, V> AsyncService<C, V> for DynService<T, Async, C, V>
where
    // S: ServiceType + 'static,
    // S::Callable<T::State, C, V>: crate::AsyncMethodCallable<T::State, C, V>,
    T: AsyncStateType<V>,
    V: Value,
    for<'a> T: 'a,
    for<'a> V: 'a,
    for<'a> C: 'a,
{
    type Get<'a> = T::Get<'a>;
    type Set<'a> = T::Set<'a>;
    type Call<'a> = T::Call<'a, AsyncMethodCallFuture<'a, T, C, V>>;

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
    pub enum AsyncMethodCallFutureState<'a, T: AsyncStateType<V>, C, V: Value> where V: 'static {
        Init {
            methods: &'a BTreeMap<String, crate::BoxAsyncMethodCallable<'static, T::State, C, V>>,
            state: Option<&'a mut T::State>,
            ctx: Option<&'a mut C>,
            name: &'a str,
            args: Option<Arguments<V>>
        },
        Call {
            #[pin]
            future: <crate::BoxAsyncMethodCallable<'static, T::State, C, V> as crate::AsyncMethodCallable<T::State, C, V>>::Future<'a>,
        },
        Done,
    }
}

#[cfg(feature = "async")]
pin_project! {
    pub struct AsyncMethodCallFuture<'a, T: AsyncStateType<V>, C, V: Value> where V: 'static {
        #[pin]
        state: AsyncMethodCallFutureState<'a,  T, C, V>
    }
}

impl<'a, T: AsyncStateType<V>, C, V: Value> core::future::Future
    for AsyncMethodCallFuture<'a, T, C, V>
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
