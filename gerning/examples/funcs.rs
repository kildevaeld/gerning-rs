use std::convert::Infallible;

use futures_core::Future;
use gerning::{
    arguments::{Arguments, ToArguments},
    AsyncCallable, AsyncCallableExt, AsyncFunc, Callable, CallableFunc, Error, Func, FuncExt,
};

#[derive(Debug)]
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
    let action = CallableFunc::new(|ctx: &mut (), person: String| async move {
        Result::<_, Error<Value>>::Ok(format!("Hello, {}", person))
    });

    

    action.call_async(&mut (), Arguments::default());

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
