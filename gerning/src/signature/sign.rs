use super::Parameters;
use crate::traits::Value;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature<T: Value> {
    params: Parameters<T>,
    return_type: Option<T::Type>,
}

impl<T: Value> Signature<T> {
    pub fn new(params: Parameters<T>, return_type: T::Type) -> Signature<T> {
        Signature {
            params,
            return_type: Some(return_type),
        }
    }

    pub fn params(&self) -> &Parameters<T> {
        &self.params
    }

    pub fn return_type(&self) -> Option<&T::Type> {
        self.return_type.as_ref()
    }
}

impl<T: Value> Default for Signature<T> {
    fn default() -> Self {
        Signature {
            params: Parameters::default(),
            return_type: None,
        }
    }
}
