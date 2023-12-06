use super::{Arguments, ArgumentsBuilder};

pub trait ToArguments<V> {
    fn to_arguments(self) -> Arguments<V>;
}

impl<V> ToArguments<V> for () {
    fn to_arguments(self) -> Arguments<V> {
        Arguments::default()
    }
}

impl<V> ToArguments<V> for Arguments<V> {
    fn to_arguments(self) -> Arguments<V> {
        self
    }
}

macro_rules! toargs {
    ($first: ident) => {
        impl<V, $first: Into<V>> ToArguments<V> for ($first,)
        {
            fn to_arguments(self) -> Arguments<V> {
                ArgumentsBuilder::default().with(self.0).build()
            }
        }
    };
    ($first: ident $($rest: ident)*) => {
        toargs!($($rest)*);

        impl<V, $first: Into<V>, $($rest: Into<V>),*> ToArguments<V> for ($first, $($rest),*)
        {
            #[allow(non_snake_case)]
            fn to_arguments(self) -> Arguments<V> {
                let mut args = ArgumentsBuilder::default();

                let ($first, $($rest),*) = self;

                args.add($first);

                $(
                    args.add($rest);
                )*

                args.build()

            }
        }
    }
}

toargs!(T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12);
