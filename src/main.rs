//use common::luaobject::objtype::Bool;


//use core::{mem::size_of, ptr::null_mut};

//use common::luastate::statedef::LuaState;




pub mod machine;
pub mod common;




use common::state::statedef::LuaState;

use crate::machine::machdef::Machine;


pub fn test_01(state:&mut LuaState)->usize{
    let k=state.pop_bool();
    let i=state.pop_integer();
    println!("the value is {},{}",i,k);
    return 0;
}


fn main(){

    

    let state=Machine::machine().get_state();
    //d(state);
    state.push_rfunc(&test_01);
    //let f=state.pop_stack();
    //let mut x=unsafe{f.get_value()};//.val_rfunc};
    
    //let u=unsafe{x.val_rfunc.into_inner().unwrap().as_ref()};
    //u(state);
    state.push_integer(2);
    state.push_bool(true);
    Machine::machine().call(2, 0);
}
