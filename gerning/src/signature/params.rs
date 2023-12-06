use crate::traits::Value;
use alloc::{sync::Arc, vec::Vec};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Parameters<V: Value>(Option<Arc<Vec<V::Type>>>);

impl<T: Value> Default for Parameters<T> {
    fn default() -> Self {
        Parameters(None)
    }
}

impl<T: Value> Parameters<T> {
    pub const fn new() -> Parameters<T> {
        Parameters(None)
    }

    pub fn build() -> ParametersBuilder<T> {
        ParametersBuilder {
            params: Vec::default(),
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T::Type> {
        self.0.as_ref().and_then(|vec| vec.get(idx))
    }

    pub fn iter(&self) -> ParamIter<'_, T::Type> {
        ParamIter {
            iter: self.0.as_ref().map(|m| m.iter()),
        }
    }
}

pub struct ParamIter<'a, T> {
    iter: Option<core::slice::Iter<'a, T>>,
}

impl<'a, T> Iterator for ParamIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = self.iter.as_mut() {
            iter.next()
        } else {
            None
        }
    }
}

pub struct ParametersBuilder<T: Value> {
    params: Vec<T::Type>,
}

impl<T: Value> ParametersBuilder<T> {
    pub fn with(mut self, param: T::Type) -> Self {
        self.add(param);
        self
    }

    pub fn add(&mut self, param: T::Type) -> &mut Self {
        self.params.push(param);
        self
    }

    pub fn build(self) -> Parameters<T> {
        Parameters(Some(Arc::new(self.params)))
    }
}
