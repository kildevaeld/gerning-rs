use std::{collections::BTreeMap, convert::Infallible};

use futures_core::Future;
use gerning::{
    arguments::{Arguments, ToArguments},
    service::{
        AsyncMethodCallable, AsyncMethodCallableExt, AsyncService, AsyncState, SendState, Service,
        State, SyncState,
    },
    AsyncCallable, AsyncCallableExt, AsyncFunc, Callable, CallableFunc, Error, Func, FuncExt,
};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
}

#[derive(Debug)]
pub enum Type {
    String,
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(value: &'a str) -> Self {
        Value::String(value.to_string())
    }
}

impl<'a> From<&'a String> for Value {
    fn from(value: &'a String) -> Self {
        Value::String(value.clone())
    }
}

impl TryFrom<Value> for String {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => {
                panic!()
            }
        }
    }
}

impl<'a> TryFrom<&'a Value> for String {
    type Error = Infallible;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s.clone()),
            _ => {
                panic!()
            }
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = Infallible;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => {
                panic!()
            }
        }
    }
}

impl gerning::Value for Value {
    type Type = Type;

    fn get_type(&self) -> Self::Type {
        match self {
            Value::String(_) => Type::String,
        }
    }
}

impl gerning::Typed<Value> for String {
    fn get_type() -> <Value as gerning::Value>::Type {
        Type::String
    }
}

impl gerning::Typed<Value> for Value {
    fn get_type() -> <Value as gerning::Value>::Type {
        Type::String
    }
}

impl<'a> gerning::Typed<Value> for &'a str {
    fn get_type() -> <Value as gerning::Value>::Type {
        Type::String
    }
}

fn test<'a, C>(ctx: &'a mut C, _test: String) -> Result<String, Error<Value>> {
    Ok(String::from("Test func"))
}

async fn test_async(ctx: &mut (), _test: String) -> Result<String, Error<Value>> {
    Ok(String::from("Test func"))
}

async fn method<F: AsyncFunc<(), (String,)>>(func: F) {
    func.call(&mut (), (String::from("value"),)).await;
}

fn main() -> Result<(), Error<Value>> {
    let callable = test.callable();
    // let async_callable = test_async.callable();

    println!("Signature: {:?}", callable.signature());

    let ret = callable.call(&mut (), ("",).to_arguments())?;

    println!("RET: {:?}", ret);
    let action = CallableFunc::new(|ctx: &mut (), person: String| {
        to_send(async move { Result::<_, Error<Value>>::Ok(format!("Hello, {}", person)) })
    })
    .boxed();

    action.call_async(&mut (), Arguments::default());

    let mut service = gerning::service::DynService::new(SyncState::new(BTreeMap::default()));

    service.register(
        "test",
        |this: &mut BTreeMap<String, Value>, ctx: &mut (), args: Arguments<Value>| {
            this.get("state").cloned().ok_or_else(|| Error::Infallible)
        },
    );

    service.set_value("state", "What a wonderful world".into())?;

    let ret = service.call(&mut (), "test", Arguments::default())?;

    println!("RET {:?}", ret);

    let mut service =
        gerning::service::DynService::new_async_send(AsyncState::new(BTreeMap::default()));

    service.register::<TestAsync>("test", TestAsync);

    futures::executor::block_on(async move {
        service
            .set_value("state", "What a wonderful world async".into())
            .await?;

        let ret = service.call(&mut (), "test", Arguments::default()).await?;

        println!("RET {:?}", ret);

        Result::<_, Error<_>>::Ok(())
    })?;

    // async_callable.call_async(&mut (), Arguments::default());
    // let callable = test.callable();

    // // let ret = callable.call(Arguments::default())?;

    // let ret = callable.call(
    //     (Test {
    //         person: Person {
    //             name: "Test".to_string(),
    //             age: 1,
    //         },
    //     },)
    //         .to_arguments(),
    // );

    // println!(
    //     "{}",
    //     serde_json::to_string_pretty(&callable.signature().params().validator()).unwrap()
    // );

    // let args = ArgumentsBuilder::default()
    //     .with(Person {
    //         name: "World".into(),
    //         age: 6,
    //     })
    //     .build();

    // let result = futures_executor::block_on(action.call_async(args))?;

    // //let result = action.call(args)?;

    // println!("{:?}", callable.signature());

    Ok(())
}

fn to_send<F>(future: F) -> impl Future<Output = F::Output> + Send
where
    F: Future + Send,
{
    future
}

struct TestAsync;

impl<S: State<Value>, C> AsyncMethodCallable<S, C, Value> for TestAsync {
    type Future<'a> = core::future::Ready<Result<Value, Error<Value>>>
    where
        Self: 'a,
        C: 'a,
        S: 'a;

    fn signature(&self) -> gerning::signature::Signature<Value> {
        todo!()
    }

    fn call_async<'a>(
        &'a self,
        this: &'a mut S,
        ctx: &'a mut C,
        args: Arguments<Value>,
    ) -> Self::Future<'a> {
        core::future::ready(Ok(this.get("state").unwrap().unwrap()))
    }
}
