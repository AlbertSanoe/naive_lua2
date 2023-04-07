#[derive(Debug, Clone, Copy)]
pub enum LuaStateStatus {
    LuaOk = 0,
    LuaErrErr = 1,
    LuaErrMem = 2, // failed allocating memory
    LuaErrRun = 3,
} // R[0-3] &15

#[derive(Debug, Clone, Copy)]
pub enum LuaCallInfoStatus {
    CallOk = 0,
    TooManyCall = 1,
    StackOverFlow = 2,
} // R[4-7] &(15<<4)

#[derive(Debug, Clone, Copy)]
pub enum ErrCode {
    Fine = 0,
    NullPointer = 1,
    NoneObject = 2,
    OverFlow = 3,
    //MisMatch,
} // R[8-11] &(15<<8)

impl Default for LuaCallInfoStatus {
    fn default() -> Self {
        LuaCallInfoStatus::CallOk
    }
}

impl Default for LuaStateStatus {
    fn default() -> Self {
        LuaStateStatus::LuaOk
    }
}

pub const LUA_MIN_STACK: u32 = 20; // for callinfo structure
pub const LUA_STACK_SIZE: u32 = 2 * LUA_MIN_STACK; // initial stack size
pub const LUA_EXTRA_STACK: u32 = 5;
pub const LUA_MAX_STACK: u32 = 15000;
pub const LUA_ERROR_STACK: u32 = 200;

pub const LUA_MUL_RET: isize = -1;
pub const LUA_MAX_CALLS: usize = 200;
pub const LUA_CI_LEN: usize = 10; // need not pop out

#[allow(unused_macros)]
macro_rules! cast {
    ($t:ident,$exp:expr) => {
        ((t)(exp))
    };
}

#[allow(unused_macros)]
macro_rules! savestack {
    ($L:ident,$o:ident) => {
        ((o)-(L)->stack)
    };
}

#[allow(unused_macros)]
macro_rules! restorestack {
    ($L:ident,$o:ident) => {
        (L->stack + (o))
    };
}
