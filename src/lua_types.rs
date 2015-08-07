pub enum LuaType{
    LString(String),
    LNumber(i32),
    LFunction,
    LBool,
    LThread,
    LTable,
    LNil
}