use mlua::{MetaMethod, UserData};

pub struct LuaCallable<C>(C);

impl<C> UserData for LuaCallable<C> {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(
            MetaMethod::Call,
            |vm, this, args: mlua::Variadic<mlua::Value>| Ok(()),
        )
    }
}
