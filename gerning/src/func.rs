pub trait Func<T> {
    type Output;
    fn call(&self, input: T) -> Self::Output;
}

impl<F, U> Func<()> for F
where
    F: Fn() -> U,
{
    type Output = U;

    fn call(&self, _ctx: ()) -> Self::Output {
        (self)()
    }
}

macro_rules! funcs {
    ($first: ident) => {
        impl< F, U, $first> Func<($first,)> for F
        where
            F: Fn($first) -> U,
        {
            type Output = U;
            fn call(&self, input: ($first,)) -> Self::Output {
               (self)(input.0)
            }
        }
    };
    ($first: ident $($rest: ident)*) => {
        funcs!($($rest)*);

        impl< F, U, $first, $($rest),*> Func<($first, $($rest),*)> for F
        where
            F: Fn($first, $($rest),*) -> U,
        {
            type Output = U;
            fn call(&self, input: ($first, $($rest),*)) -> Self::Output {
                #[allow(non_snake_case)]
                let ($first, $($rest),*) = input;
                (self)($first, $($rest),*)
            }
        }

    };
}

funcs!(T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16);
