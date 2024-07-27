use super::{error::ArgumentError, Arguments};
use crate::{
    signature::Parameters,
    traits::{Typed, Value},
};
use core::convert::Infallible;

pub trait FromArguments<'a, T: Value>: Sized + Send {
    type Error: Into<ArgumentError<T>>;
    fn from_arguments(args: &'a mut Arguments<T>) -> Result<Self, Self::Error>;

    fn parameters() -> Parameters<T>;
}

impl<'a, T: Value> FromArguments<'a, T> for () {
    type Error = Infallible;
    fn from_arguments(_args: &'a mut Arguments<T>) -> Result<Self, Self::Error> {
        Ok(())
    }

    fn parameters() -> Parameters<T> {
        Parameters::default()
    }
}

macro_rules! count {
    (@step $idx: expr, $args:expr, $type1:ident, $( $type:ident ),*) => {

        let $type1 = $args.try_get_ref::<$type1>($idx)?;
        count!(@step $idx + 1usize, $args, $($type),*);
    };

    (@step $idx: expr, $args:expr, $type1:ident) => {
        let $type1 = $args.try_get_ref::<$type1>($idx)?;
    };

    (@step $_idx:expr, $args: expr,) => {};
}

macro_rules! arguments {
    ($first: ident) => {
        impl<'a,V: Value + 'a, $first: Typed<V> + TryFrom<&'a V> + Send> FromArguments<'a, V> for ($first,)
        where
            $first::Error: Into<ArgumentError<V>>
        {
            type Error = ArgumentError<V>;
            fn from_arguments(args: &'a mut Arguments<V>) -> Result<Self, Self::Error> {
                Ok((args.try_get_ref::<$first>(0)?,))
            }

            fn parameters() -> Parameters<V> {
                Parameters::build().with($first::get_type()).build()
            }
        }
    };

    ($first: ident $($rest: ident)*) => {

        arguments!($($rest)*);


        impl<'a, V: Value + 'a, $first: Typed<V> + TryFrom<&'a V>  + Send, $($rest: Typed<V> + TryFrom<&'a V>  + Send),*> FromArguments<'a, V> for ($first,$($rest),*)
        where
            $first::Error: Into<ArgumentError<V>>,
            $(
                $rest::Error: Into<ArgumentError<V>>,
            )*
        {
            type Error = ArgumentError<V>;
            #[allow(non_snake_case)]
            fn from_arguments(args: &'a mut Arguments<V>) -> Result<Self, Self::Error> {

                count!(@step 0, args, $first, $($rest),*);

                Ok((
                    $first, $($rest),*
                ))
            }

            fn parameters() -> Parameters<V> {
               let mut params = Parameters::build();
               params.add($first::get_type());
               $(
                params.add($rest::get_type());
               )*

               params.build()
            }
        }
    };
}

arguments!(T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16);
