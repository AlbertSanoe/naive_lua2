use std::ptr::{null_mut, NonNull};
const ILLEGAL_INDEX: usize = usize::MAX;
use crate::common::{
    lua::ErrCode,
    lua::{LuaCallInfoStatus, LuaStateStatus, LUA_MIN_STACK, LUA_MUL_RET},
    obj::{
        objdef::{TNumber, TObject, BASIC_TYPE_BIT},
        objtrait::ObjectTrait,
    },
    state::statedef::{CallInfo, LuaState, StkElem},
};

macro_rules! ptr_get {
    ($self:ident,$stack:ident) => {{
        if let Some(stk) = $self.$stack {
            let ptr = stk.as_ptr();
            if !ptr.is_null() {
                Ok(unsafe { &mut *ptr })
            } else {
                Err(ErrCode::NullPointer)
            }
        } else {
            Err(ErrCode::NoneObject)
        }
    }};
    ($self:ident,$stack:ident,$elem:ident) => {{
        if let Some(stk) = $self.$stack.$elem {
            let ptr = stk.as_ptr();
            if !ptr.is_null() {
                Ok(unsafe { &mut *ptr })
            } else {
                Err(ErrCode::NullPointer)
            }
        } else {
            Err(ErrCode::NoneObject)
        }
    }};
    ($sth:expr) => {
        if let Some(stack) = $sth {
            Ok(stack)
        } else {
            Err(ErrCode::NoneObject)
        }
    };
}

macro_rules! ptr_init {
    ($content:expr) => {
        NonNull::from($content)
    };
    ($content:ident) => {
        NonNull::new($content)
    };
    ($self:ident,$ci:ident) => {
        NonNull::from(&$self.$ci)
    };
}

#[allow(dead_code)]
pub struct Machine {
    dynamo: Routine,
    //wiper: ErrorHandler,
    ci_err_index: usize,
    cstate_status: LuaStateStatus,
}

static mut MACHINE: *mut Machine = null_mut();

impl Machine {
    pub fn machine() -> &'static mut Self {
        if unsafe { MACHINE.is_null() } {
            unsafe { MACHINE = Box::leak(Box::new(Machine::new())) };
            unsafe {
                return &mut *MACHINE;
            }
        } else {
            return unsafe { &mut *MACHINE };
        }
    }

    pub fn get_state(&mut self) -> &mut LuaState {
        unsafe { self.dynamo.cstate.unwrap().as_mut() }
    }

    #[allow(dead_code)]
    fn new() -> Self {
        // start the machine
        let mut machine = Machine {
            dynamo: Routine::new(),
            //wiper: Default::default(),
            ci_err_index: ILLEGAL_INDEX,
            cstate_status: LuaStateStatus::LuaOk,
        };

        // generate the states
        let state = LuaState::mainthread_new(null_mut());
        // null_mut => for temp

        // link the state to dynamo
        machine.dynamo.cstate = Some(ptr_init!(state));

        machine
    }

    pub fn call(&mut self, narg: usize, sresults: isize) -> usize {
        let state = ptr_get!(self, dynamo, cstate).ok().unwrap();
        let func_index = state.get_top_index() - (narg + 1);
        dbg!(state.get_top_index());
        dbg!(func_index);
        let status = self.execute(func_index, sresults);
        return status;
    }

    // INTERFACE
    #[allow(dead_code)]
    pub fn execute(&mut self, func_index: usize, sresults: isize) -> usize {
        self.execute_unprotected(func_index, sresults);
        let state = ptr_get!(self, dynamo, cstate).ok().unwrap();
        let last = state.pop_stack();
        if last.get_type() == TNumber::NumInt as u8 {
            let outcome = unsafe { last.get_value().val_int }.into_inner().unwrap();
            if outcome != 0 {
                println!("Virtual Machine: Failed running..");
            }
            //lack sth
            else {
                println!("Virtual Machine: Successful running..");
            }
        } else {
            panic!("FATAL ERROR!");
        }
        println!("The program is completed.");
        return 1;
    }

    pub fn execute_unprotected(&mut self, func_index: usize, sresults: isize) {
        // try to run here, if it does not work, back to here
        let res = self.dynamo.run(func_index, sresults);
        let state = ptr_get!(self, dynamo, cstate).ok().unwrap();
        if let Err(ecode) = res {
            state.move_top_to(func_index);

            state.stack_shrink(self.dynamo.ci_err_index);
            state.civ_shrink(self.dynamo.ci_err_index);

            let code1 = state.get_status() as isize;
            let code2 = state.get_ci_status(self.dynamo.ci_err_index) as isize;
            let code3 = ecode as isize;
            let code = (code3 << 8) | (code2 << 4) | code1;
            state.push_errcode(code as i32);
        } else {
            state.push_errcode(0);
        }
    }

    //pub fn error() {}
}

#[allow(dead_code)]
pub struct Routine {
    ci_err_index: usize,
    cci_index: usize,
    cci_status: LuaCallInfoStatus,
    cstate: Option<NonNull<LuaState>>,
}

impl Routine {
    pub fn get_state(&mut self) -> &mut LuaState {
        ptr_get!(self, cstate).ok().unwrap()
    }

    fn new() -> Self {
        Self {
            ci_err_index: ILLEGAL_INDEX,
            cci_index: 0, // as cache
            cci_status: LuaCallInfoStatus::CallOk,
            cstate: Default::default(),
        }
    }

    fn run(&mut self, func_index: usize, sresults: isize) -> Result<ErrCode, ErrCode> {
        let state = ptr_get!(self, cstate).ok().unwrap();

        if !state.calls_check() {
            self.ci_err_index = self.cci_index;
            state.write_ci_status(self.cci_index, LuaCallInfoStatus::TooManyCall);
            state.set_status(LuaStateStatus::LuaErrErr);
            return Err(ErrCode::OverFlow);
        }

        // add ncalls
        state.change_ncalls(1, true);

        // after entering precall function, no more try catch block
        self.pre_call(func_index, sresults)?;

        // minus ncalls
        state.change_ncalls(1, false);

        Ok(ErrCode::Fine)
    }

    // prepare for function call.
    // if we call a c function, just directly call it
    // if we call a lua function, the function is just for preparation
    fn pre_call(&mut self, func_index: usize, sresults: isize) -> Result<ErrCode, ErrCode> {
        // get the current state
        let state = ptr_get!(self, cstate).ok().unwrap();

        // get the current stack
        let stack = ptr_get!(state.get_stack_mut_ref()).ok().unwrap();

        let obj = ptr_get!(stack.get_ref_elem(func_index)).ok().unwrap();

        // function label
        let label = obj.get_type();
        dbg!(label);
        // mismatched type
        if !TObject::is_function(label) {
            panic!("FATAL ERROR: Mismatch Type!");
        }
        dbg!("hello", label >> BASIC_TYPE_BIT);
        match label >> BASIC_TYPE_BIT {
            1 => {
                dbg!();
                let function = unsafe { obj.get_value().val_rfunc.into_inner().unwrap().as_ref() };
                dbg!();
                //let function
                dbg!();
                //dbg!(function as *const ());
                //dbg!(test_01 as *const());
                // checking stack status and resize it silently
                state.stack_check(LUA_MIN_STACK as usize);
                dbg!();
                // add a new call info, the info of cci in state changes as well
                self.cci_index = state.add_next_ci(func_index, sresults);
                dbg!();
                //let function=unsafe{&mut *function};
                //dbg!("2 times",function as *const ());
                // if !function.is_null(){
                //     dbg!("fuck you");
                // }
                let rresults = function(state);
                dbg!();
                dbg!(rresults);

                let _ = state.pop_stack();
                if !state.cci_check(self.cci_index, rresults) {
                    self.ci_err_index = self.cci_index;
                    state.write_ci_status(self.cci_index, LuaCallInfoStatus::StackOverFlow);
                    state.set_status(LuaStateStatus::LuaErrErr);
                    return Err(ErrCode::OverFlow);
                }

                self.post_call(func_index, rresults, sresults);

                self.cci_index -= 1;

                return Ok(ErrCode::Fine);
            }
            _ => {
                // self.ci_err_index = self.cci_index;
                // return Err(ErrCode::MisMatch);
                panic!("Mismatch Type!");
            }
        }
    }

    fn post_call(&self, func_index: usize, rresults: usize, sresults: isize) {
        let state = ptr_get!(self, cstate).ok().unwrap();
        let stack = ptr_get!(state.get_stack_mut_ref()).ok().unwrap();

        match sresults {
            0 => {
                state.move_top_to(func_index);
                // not really necessary, just in case
            }
            1 => {
                if rresults == 0 {
                    let mut empty_elem = StkElem::default();
                    let _ = stack.swap_elem(func_index, &mut empty_elem).ok().unwrap();
                }
                state.move_top_to(func_index + 1);
            }
            LUA_MUL_RET => {
                todo!()
            } //
            _ => panic!("dd"),
        } // inside match, deal with stack

        let mut callinfo = CallInfo::default();
        let civ = ptr_get!(state.get_civ_mut_ref()).ok().unwrap();
        let _ = civ.swap_elem(self.cci_index, &mut callinfo).ok().unwrap();
        // deal with call info
    }
}
