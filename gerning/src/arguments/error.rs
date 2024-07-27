use core::{convert::Infallible, fmt};

use crate::traits::Value;

#[derive(Debug)]
pub enum ArgumentError<T: Value> {
    Infallible,
    IvalidType { expected: T::Type, found: T::Type },
    Missing { index: usize, arity: usize },
    IndexOutOfBounds(usize),
}

impl<T: Value> fmt::Display for ArgumentError<T>
where
    T::Type: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgumentError::Infallible => {
                write!(f, "infallible")
            }
            ArgumentError::IvalidType { expected, found } => {
                write!(f, "invalid type. Expected: {expected:?}, found: {found:?}")
            }
            ArgumentError::Missing { index, .. } => {
                write!(f, "missing argument at index: {index:}")
            }
            ArgumentError::IndexOutOfBounds(idx) => {
                write!(f, "index out of bounds: {idx}")
            }
        }
    }
}

impl<T: Value> From<Infallible> for ArgumentError<T> {
    fn from(_: Infallible) -> Self {
        ArgumentError::Infallible
    }
}
