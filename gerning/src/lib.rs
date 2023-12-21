#![no_std]

extern crate alloc;

mod callable;
#[cfg(feature = "async")]
mod callable_async;
mod callable_fn;
mod error;
mod func;
mod resultable;
mod traits;

pub mod arguments;
pub mod signature;

pub use self::{callable::*, callable_fn::*, error::*, func::*, resultable::*, traits::*};

#[cfg(feature = "async")]
pub use self::callable_async::*;
