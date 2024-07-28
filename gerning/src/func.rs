pub trait Func<C, T> {
    type Output;

    fn call(&self, ctx: &mut C, input: T) -> Self::Output;
}

impl<F, C, U> Func<C, ()> for F
where
    F: Fn(&mut C) -> U + 'static,
{
    type Output = U;

    fn call(&self, ctx: &mut C, _arg: ()) -> Self::Output {
        (self)(ctx)
    }
}

macro_rules! funcs {
    ($first: ident) => {
        impl< F, C, U, $first> Func<C, ($first,)> for F
        where
            F: 'static,
            F: Fn(&mut C, $first) -> U,
            // for< U: 'a,
            // for< C: 'a
        {
            type Output = U;
            fn call(& self, ctx: & mut C, input: ($first,)) -> Self::Output {
               (self)(ctx, input.0)
            }
        }
    };
    ($first: ident $($rest: ident)*) => {
        funcs!($($rest)*);

        impl< F, C, U, $first, $($rest),*> Func<C, ($first, $($rest),*)> for F
        where
             F: Fn(&mut C, $first, $($rest),*) -> U + 'static,


        {
            type Output = U;
            fn call(& self,ctx: & mut C, input: ($first, $($rest),*)) -> Self::Output {
                #[allow(non_snake_case)]
                let ($first, $($rest),*) = input;
                (self)(ctx,$first, $($rest),*)
            }
        }

    };
}

funcs!(T1 T2 T3 T4 T5 T6 T7 T8);

#[cfg(feature = "async")]
mod async_impl {
    use futures_core::Future;

    pub trait AsyncFunc<C, T> {
        type Output;
        type Future<'a>: Future<Output = Self::Output> + 'a
        where
            Self: 'a,
            C: 'a;

        fn call<'a>(&'a self, ctx: &'a mut C, input: T) -> Self::Future<'a>;
    }

    impl<F, C, U> AsyncFunc<C, ()> for F
    where
        F: 'static,
        for<'a> F: Fn(&'a mut C) -> U,
        U: Future,
        for<'a> U: 'a,
    {
        type Output = U::Output;
        type Future<'a> = U where  C: 'a;

        fn call<'a>(&'a self, ctx: &'a mut C, _input: ()) -> Self::Future<'a> {
            (self)(ctx)
        }
    }

    macro_rules! funcs {
        ($first: ident) => {
            impl< F, C, U, $first> AsyncFunc<C, ($first,)> for F
            where
                F: 'static,
                F: Fn(&mut C, $first) -> U,
                U: Future,
                for<'a> U: 'a
                // for< U: 'a,
                // for< C: 'a
            {
                type Output = U::Output;
                type Future<'a> = U where C: 'a;
                fn call<'a>(&'a  self, ctx: &'a mut C, input: ($first,)) -> Self::Future<'a> {
                   (self)(ctx, input.0)
                }
            }
        };
        ($first: ident $($rest: ident)*) => {
            funcs!($($rest)*);

            impl< F, C, U, $first, $($rest),*> AsyncFunc<C, ($first, $($rest),*)> for F
            where
                 F: Fn(&mut C, $first, $($rest),*) -> U + 'static,
                 U: Future,
                 for<'a> U: 'a


            {
                type Output = U::Output;
                type Future<'a> = U where C:'a;
                fn call<'a>(&'a self,ctx: &'a mut C, input: ($first, $($rest),*)) -> Self::Future<'a> {
                    #[allow(non_snake_case)]
                    let ($first, $($rest),*) = input;
                    (self)(ctx,$first, $($rest),*)
                }
            }

        };
    }

    funcs!(T1 T2 T3 T4 T5 T6 T7 T8);
}

#[cfg(feature = "async")]
pub use async_impl::*;

#[cfg(all(feature = "service", feature = "async"))]
mod async_method_impl {
    use futures_core::Future;

    pub trait AsyncMethod<S, C, T> {
        type Output;
        type Future<'a>: Future<Output = Self::Output> + 'a
        where
            Self: 'a,
            S: 'a,
            C: 'a;

        fn call<'a>(&'a self, this: &'a mut S, ctx: &'a mut C, input: T) -> Self::Future<'a>;
    }

    impl<F, S, C, U> AsyncMethod<S, C, ()> for F
    where
        F: 'static,
        for<'a> F: Fn(&'a mut S, &'a mut C) -> U,
        U: Future,
        for<'a> U: 'a,
    {
        type Output = U::Output;
        type Future<'a> = U where  C: 'a, S:'a;

        fn call<'a>(&'a self, this: &'a mut S, ctx: &'a mut C, _input: ()) -> Self::Future<'a> {
            (self)(this, ctx)
        }
    }

    macro_rules! funcs {
        ($first: ident) => {
            impl< F, S, C, U, $first> AsyncMethod<S, C, ($first,)> for F
            where
                F: 'static,
                F: Fn(&mut S, &mut C, $first) -> U,
                U: Future,
                for<'a> U: 'a,
                for<'a> S: 'a,
                // for< U: 'a,
                // for< C: 'a
            {
                type Output = U::Output;
                type Future<'a> = U where C: 'a;
                fn call<'a>(&'a self, this: &'a mut S, ctx: &'a mut C, input: ($first,)) -> Self::Future<'a> {
                   (self)(this, ctx, input.0)
                }
            }
        };
        ($first: ident $($rest: ident)*) => {
            funcs!($($rest)*);

            impl< F, S, C, U, $first, $($rest),*> AsyncMethod<S, C, ($first, $($rest),*)> for F
            where
                 F: Fn(&mut S, &mut C, $first, $($rest),*) -> U + 'static,
                 U: Future,
                 for<'a> U: 'a


            {
                type Output = U::Output;
                type Future<'a> = U where C:'a, S:'a;
                fn call<'a>(&'a self, this: &'a mut S, ctx: &'a mut C, input: ($first, $($rest),*)) -> Self::Future<'a> {
                    #[allow(non_snake_case)]
                    let ($first, $($rest),*) = input;
                    (self)(this,ctx,$first, $($rest),*)
                }
            }

        };
    }

    funcs!(T1 T2 T3 T4 T5 T6 T7 T8);
}

#[cfg(all(feature = "service", feature = "async"))]
pub use async_method_impl::*;

// #[cfg(all(feature = "service", not(feature = "async")))]
// mod method_impl {

//     pub trait Method<C, T> {
//         type Output;
//         type Future<'a>: Future<Output = Self::Output> + 'a
//         where
//             Self: 'a,
//             C: 'a;

//         fn call<'a>(&'a self, ctx: &'a mut C, input: T) -> Self::Future<'a>;
//     }

//     impl<F, C, U> Method<C, ()> for F
//     where
//         F: 'static,
//         for<'a> F: Fn(&'a mut C) -> U,
//         U: Future,
//         for<'a> U: 'a,
//     {
//         type Output = U::Output;
//         type Future<'a> = U where  C: 'a;

//         fn call<'a>(&'a self, ctx: &'a mut C, _input: ()) -> Self::Future<'a> {
//             (self)(ctx)
//         }
//     }

//     macro_rules! funcs {
//         ($first: ident) => {
//             impl< F, C, U, $first> Method<C, ($first,)> for F
//             where
//                 F: 'static,
//                 F: Fn(&mut C, $first) -> U,
//                 U: Future,
//                 for<'a> U: 'a
//                 // for< U: 'a,
//                 // for< C: 'a
//             {
//                 type Output = U::Output;
//                 type Future<'a> = U where C: 'a;
//                 fn call<'a>(&'a  self, ctx: &'a mut C, input: ($first,)) -> Self::Future<'a> {
//                    (self)(ctx, input.0)
//                 }
//             }
//         };
//         ($first: ident $($rest: ident)*) => {
//             funcs!($($rest)*);

//             impl< F, C, U, $first, $($rest),*> Method<C, ($first, $($rest),*)> for F
//             where
//                  F: Fn(&mut C, $first, $($rest),*) -> U + 'static,
//                  U: Future,
//                  for<'a> U: 'a

//             {
//                 type Output = U::Output;
//                 type Future<'a> = U where C:'a;
//                 fn call<'a>(&'a self,ctx: &'a mut C, input: ($first, $($rest),*)) -> Self::Future<'a> {
//                     #[allow(non_snake_case)]
//                     let ($first, $($rest),*) = input;
//                     (self)(ctx,$first, $($rest),*)
//                 }
//             }

//         };
//     }

//     funcs!(T1 T2 T3 T4 T5 T6 T7 T8);
// }

// #[cfg(all(feature = "service", not(feature = "async")))]
// pub use method_impl::*;
