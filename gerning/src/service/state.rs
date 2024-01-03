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

use crate::{arguments::Arguments, Error, Value};

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
