use crate::{Error, Value};
use alloc::{
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    sync::Arc,
};
use avagarden::sync::Mutex;
use core::cell::RefCell;
#[cfg(feature = "async")]
use futures_core::Future;
#[cfg(feature = "async")]
use locket::AsyncLockApi;
use locket::{LockApi, LockApiWriteGuard, LockError};

pub trait HasState {
    type State;
}

pub trait StateType<V: Value>: HasState {
    type Ref<'a>: LockApiWriteGuard<'a, Self::State>
    where
        Self: 'a,
        Self::State: 'a;

    fn get<'a>(&'a self) -> Result<Self::Ref<'a>, LockError>;
}

#[cfg(feature = "async")]
pub trait AsyncStateType<V: Value>: HasState {
    type Ref<'a>: LockApiWriteGuard<'a, Self::State>
    where
        Self: 'a,
        Self::State: 'a;
    type Future<'a>: Future<Output = Result<Self::Ref<'a>, LockError>>
    where
        Self: 'a;

    fn get<'a>(&'a self) -> Self::Future<'a>;
}

pub struct SendState<T> {
    state: Arc<Mutex<T>>,
}

unsafe impl<T: Send> Send for SendState<T> {}

unsafe impl<T: Send> Sync for SendState<T> {}

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
impl<T: State<V> + 'static, V: Value> AsyncStateType<V> for SendState<T> {
    type Ref<'a>: = avagarden::sync::MutexGuard<'a, T>
    where
        Self: 'a,
        Self::State: 'a;
    type Future<'a> = core::future::Ready<Result<Self::Ref<'a>, LockError>> where Self: 'a, T: 'a;

    fn get<'a>(&'a self) -> Self::Future<'a> {
        core::future::ready(<Arc<Mutex<T>> as LockApi<T>>::write(&self.state))
    }
}

pub struct AsyncState<T> {
    state: Arc<async_lock::Mutex<T>>,
}

unsafe impl<T: Send> Send for AsyncState<T> {}

unsafe impl<T: Send> Sync for AsyncState<T> {}

impl<T> HasState for AsyncState<T> {
    type State = T;
}

impl<T> AsyncState<T> {
    pub fn new(state: T) -> AsyncState<T> {
        AsyncState {
            state: Arc::new(async_lock::Mutex::new(state)),
        }
    }
}

#[cfg(feature = "async")]
impl<T: State<V> + 'static + Send, V: Value> AsyncStateType<V> for AsyncState<T> {
    type Ref<'a>: = async_lock::MutexGuard<'a, T>
    where
        Self: 'a,
        Self::State: 'a;
    type Future<'a> = locket::FutureResult<async_lock::futures::Lock<'a, T>> where Self: 'a, T: 'a;

    fn get<'a>(&'a self) -> Self::Future<'a> {
        <Arc<async_lock::Mutex<T>> as AsyncLockApi<T>>::write(&self.state)
    }
}

//
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

impl<T: State<V> + 'static, V: Value> StateType<V> for SyncState<T> {
    type Ref<'a> = core::cell::RefMut<'a, T> where Self: 'a, T: 'a;

    fn get<'a>(&'a self) -> Result<Self::Ref<'a>, LockError> {
        <RefCell<T> as LockApi<T>>::write(&*self.state)
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
