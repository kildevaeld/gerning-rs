#[cfg(feature = "async")]
use crate::callable_async::AsyncCallable;
use core::marker::PhantomData;
#[cfg(feature = "async")]
use core::pin::Pin;
#[cfg(feature = "async")]
use pin_project_lite::pin_project;

use crate::{
    arguments::{Arguments, FromArguments},
    func::Func,
    signature::Signature,
    traits::{Typed, Value},
    Callable, Error, Resultable,
};

pub struct CallableFunc<F, C, A, V> {
    func: F,
    _args: PhantomData<(C, A, V)>,
}

impl<F: Clone, C, A, V> Clone for CallableFunc<F, C, A, V> {
    fn clone(&self) -> Self {
        CallableFunc {
            func: self.func.clone(),
            _args: PhantomData,
        }
    }
}

impl<F: Copy, C, A, V> Copy for CallableFunc<F, C, A, V> {}

unsafe impl<F: Send, C, A, V> Send for CallableFunc<C, F, A, V> {}

unsafe impl<F: Sync, C, A, V> Sync for CallableFunc<C, F, A, V> {}

impl<F, C, A, V: Value> CallableFunc<F, C, A, V>
where
    for<'a> A: FromArguments<'a, V>,
{
    pub fn new(func: F) -> Self
    where
        F: crate::func::Func<C, A>,
    {
        CallableFunc {
            func,
            _args: PhantomData,
        }
    }
}

impl<F, C, A, V: Value> Callable<C, V> for CallableFunc<F, C, A, V>
where
    for<'a> A: FromArguments<'a, V>,
    F: crate::func::Func<C, A>,
    F::Output: Resultable,
    <F::Output as Resultable>::Ok: Into<V> + Typed<V>,
    <F::Output as Resultable>::Error: Into<Error<V>>,
{
    fn signature(&self) -> Signature<V> {
        Signature::new(
            A::parameters(),
            <<F::Output as Resultable>::Ok as Typed<V>>::get_type(),
        )
    }

    fn call<'a>(&self, ctx: &'a mut C, args: Arguments<V>) -> Result<V, Error<V>> {
        let args = A::from_arguments(&args).map_err(|err| err.into())?;

        Ok(self
            .func
            .call(ctx, args)
            .into_result()
            .map_err(Into::into)?
            .into())
    }
}

#[cfg(feature = "async")]
impl<F, C, A, V: Value + 'static> AsyncCallable<C, V> for CallableFunc<F, C, A, V>
where
    for<'a> A: FromArguments<'a, V> + 'a,
    // for<'a> C: 'a,
    F: crate::func::AsyncFunc<C, A> + 'static,
    // for<'a> F::Output<'a>: Future + 'a,
    F::Output: Resultable,
    <F::Output as Resultable>::Error: Into<Error<V>>,
    <F::Output as Resultable>::Ok: Into<V> + Typed<V>,
{
    type Future<'a> = CallableFuncFuture<'a, F::Future<'a>, V> where C: 'a;

    fn signature(&self) -> Signature<V> {
        Signature::new(
            A::parameters(),
            <<F::Output as Resultable>::Ok as Typed<V>>::get_type(),
        )
    }

    fn call_async<'a>(&'a self, ctx: &'a mut C, args: Arguments<V>) -> Self::Future<'a> {
        let state = match A::from_arguments(&args).map_err(|err| err.into()) {
            Err(err) => CallableFuncFutureState::Error {
                error: Some(err.into()),
            },
            Ok(args) => CallableFuncFutureState::Future {
                future: self.func.call(ctx, args),
            },
        };

        CallableFuncFuture {
            func: state,
            lifetime: core::marker::PhantomData,
        }
    }
}

#[cfg(feature = "async")]
pin_project! {
    #[project = EnumProj]
    enum CallableFuncFutureState<F, V: Value> {
        Error {
            error: Option<Error<V>>
        },
        Future {
            #[pin]
            future: F
        }
    }
}

#[cfg(feature = "async")]
pin_project! {
    pub struct CallableFuncFuture<'a, F, V: Value> {
        #[pin]
        func: CallableFuncFutureState<F, V>,
        lifetime: core::marker::PhantomData<&'a V>
    }
}

#[cfg(feature = "async")]
unsafe impl<'a, F: Send, V: Value> Send for CallableFuncFuture<'a, F, V> {}

#[cfg(feature = "async")]
impl<'a, F, V: Value> core::future::Future for CallableFuncFuture<'a, F, V>
where
    F: core::future::Future + 'a,
    F::Output: Resultable,
    <F::Output as Resultable>::Error: Into<Error<V>>,
    <F::Output as Resultable>::Ok: Into<V> + Typed<V>,
{
    type Output = Result<V, Error<V>>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        use core::task::Poll;
        let this = self.as_mut().project();

        match this.func.project() {
            EnumProj::Error { error } => Poll::Ready(Err(error.take().expect("call after finish"))),
            EnumProj::Future { future } => match future.poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(ret) => match ret.into_result() {
                    Ok(ret) => Poll::Ready(Ok(ret.into())),
                    Err(err) => Poll::Ready(Err(err.into())),
                },
            },
        }
    }
}

pub trait FuncExt<C, A>: Func<C, A> {
    fn callable<V: Value>(self) -> CallableFunc<Self, C, A, V>
    where
        Self: Sized,
        for<'a> A: FromArguments<'a, V>,
    {
        CallableFunc::new(self)
    }
}

impl<F, C, A> FuncExt<C, A> for F where F: Func<C, A> {}
