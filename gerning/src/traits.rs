use core::fmt::Debug;

pub trait Value: Debug {
    type Type: Debug;

    fn get_type(&self) -> Self::Type;
}

pub trait Typed<T: Value> {
    fn get_type() -> T::Type;
}
