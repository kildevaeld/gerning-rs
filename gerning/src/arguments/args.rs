use crate::traits::Value;

use super::error::ArgumentError;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arguments<V> {
    args: Vec<V>,
}

impl<V> Default for Arguments<V> {
    fn default() -> Self {
        Arguments {
            args: Vec::default(),
        }
    }
}

impl<V> Arguments<V> {
    pub fn new(args: Vec<V>) -> Arguments<V> {
        Arguments { args }
    }
}

impl<T: Value> Arguments<T> {
    pub fn try_get_ref<'a, V: TryFrom<&'a T>>(&'a self, idx: usize) -> Result<V, ArgumentError<T>>
    where
        V::Error: Into<ArgumentError<T>>,
    {
        let val = match self.args.get(idx) {
            Some(ret) => ret,
            None => {
                return Err(ArgumentError::Missing {
                    index: idx,
                    arity: self.args.len(),
                })
            }
        };

        V::try_from(val).map_err(|err| err.into())
    }

    // pub fn try_get<V: TryFrom<T>>(&self, idx: usize) -> Result<V, ArgumentError<T>>
    // where
    //     V::Error: Into<ArgumentError<T>>,
    // {
    //     let val = match self.args.get(idx) {
    //         Some(ret) => ret,
    //         None => {
    //             return Err(ArgumentError::Missing {
    //                 index: idx,
    //                 arity: self.args.len(),
    //             })
    //         }
    //     };

    //     V::try_from(val.clone()).map_err(|err| err.into())
    // }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.args.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.args.get_mut(idx)
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn types(&self) -> Vec<T::Type> {
        self.args.iter().map(|m| m.get_type()).collect()
    }
}

impl<T> IntoIterator for Arguments<T> {
    type IntoIter = alloc::vec::IntoIter<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

#[derive(Debug)]
pub struct ArgumentsBuilder<V> {
    args: Vec<V>,
}

impl<V> Default for ArgumentsBuilder<V> {
    fn default() -> Self {
        ArgumentsBuilder {
            args: Vec::default(),
        }
    }
}

impl<T> ArgumentsBuilder<T> {
    pub fn with<V: Into<T>>(mut self, value: V) -> Self {
        self.args.push(value.into());
        self
    }

    pub fn add<V: Into<T>>(&mut self, value: V) -> &mut Self {
        self.args.push(value.into());
        self
    }

    pub fn build(self) -> Arguments<T> {
        Arguments { args: self.args }
    }
}
