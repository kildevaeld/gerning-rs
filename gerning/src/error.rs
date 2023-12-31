use core::fmt::Debug;

use crate::{arguments::ArgumentError, traits::Value};
use alloc::{boxed::Box, fmt};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error<V: Value> {
    Argument(ArgumentError<V>),
    Runtime(Box<dyn core::fmt::Debug + Send + Sync>),
    #[cfg(feature = "service")]
    MethodNotFound,
    #[cfg(feature = "service")]
    Lock,
    Infallible,
}

impl<V: Value> Error<V> {
    pub fn new<E: Into<Box<dyn core::fmt::Debug + Send + Sync>>>(error: E) -> Error<V> {
        Error::Runtime(error.into())
    }
}

impl<V: Value> From<ArgumentError<V>> for Error<V> {
    fn from(value: ArgumentError<V>) -> Self {
        Error::Argument(value)
    }
}

impl<V: Value + Debug> From<core::convert::Infallible> for Error<V> {
    fn from(_value: core::convert::Infallible) -> Self {
        Error::Infallible
    }
}

impl<V: Value> fmt::Display for Error<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Argument(a) => write!(f, "{}", a),
            Error::Runtime(e) => e.fmt(f),
            #[cfg(feature = "service")]
            Error::MethodNotFound => write!(f, "method not found"),
            Error::Infallible => write!(f, "infallible"),
            #[cfg(feature = "service")]
            Error::Lock => write!(f, "lock"),
        }
    }
}

#[cfg(feature = "service")]
impl<V: Value> From<locket::LockError> for Error<V> {
    fn from(_value: locket::LockError) -> Self {
        Error::Lock
    }
}
