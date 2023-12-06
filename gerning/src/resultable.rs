pub trait Resultable {
    type Ok;
    type Error;
    fn into_result(self) -> Result<Self::Ok, Self::Error>;
}

impl<T, E> Resultable for Result<T, E> {
    type Ok = T;
    type Error = E;

    fn into_result(self) -> Result<T, E> {
        self
    }
}

macro_rules! impl_result {
    ($($ty: ty)+) => {
        $(
            impl Resultable for $ty {
                type Ok = $ty;
                type Error = core::convert::Infallible;
                fn into_result(self) -> Result<Self::Ok, Self::Error> {
                    Ok(self)
                }
            }
        )+
    };
}

impl_result!(u8 i8 u16 i16 u32 i32 u64 i64 f32 f64 bool alloc::string::String);
