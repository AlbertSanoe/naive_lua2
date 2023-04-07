use core::cell::UnsafeCell;
use core::mem::{size_of, swap};
use core::ptr::null_mut;
use core::ptr::NonNull;

use crate::common::lua::{ErrCode, LUA_MAX_CALLS};
use crate::common::lua::{LuaCallInfoStatus, LuaStateStatus};
use crate::common::lua::{LUA_CI_LEN, LUA_MAX_STACK};
use crate::common::lua::{LUA_EXTRA_STACK, LUA_MIN_STACK, LUA_STACK_SIZE};

use crate::common::obj::objdef::{TNumber, TObj, TObject};
use crate::common::obj::objtrait::ObjectTrait;
use crate::common::obj::objtype::{FLT, INT, RFUNC};

const ILLEGAL_INDEX: usize = usize::MAX;

pub type StkElem = TObj;

/// brief: a macro to get the pointer
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
    ($sth:expr) => {
        if let Some(stack) = $sth {
            Ok(stack)
        } else {
            Err(ErrCode::NoneObject)
        }
    };
    ($sth:expr,$t:ty) => {
        &mut $sth
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

static mut META: *mut Meta = null_mut();

#[derive(Default,Debug)]
#[allow(dead_code)]
struct GlobalState {
    mainthread: Option<NonNull<LuaState>>,
    userdata: Option<NonNull<()>>,
}

#[derive(Default,Debug)]
#[allow(dead_code)]
struct Base {
    pub extra: [char; LUA_EXTRASPACE],
    pub state: LuaState,
}

#[derive(Default,Debug)]
#[allow(dead_code)]
struct Meta {
    pub base: Base,
    pub global: GlobalState,
}

#[inline(always)]
#[allow(dead_code)]
fn get_meta_mut() -> Option<&'static mut Meta> {
    if !unsafe { META.is_null() } {
        unsafe { Some(&mut *META) }
    } else {
        None
    }
}
#[macro_export]
macro_rules! get_meta {
    () => {
        if let Some(meta) = get_meta_mut() {
            Ok(meta)
        } else {
            Err(ErrCode::NoneObject)
        }
    };
}
#[macro_export]
macro_rules! get_global_state {
    () => {
        get_meta!().ok().unwrap().global
    };
}

#[macro_export]
macro_rules! get_main_state {
    () => {
        get_meta!().ok().unwrap().base.state
    };
}

impl Meta {
    pub fn new() {
        unsafe { META = Box::leak(Box::new(Meta::default())) };

        unsafe{dbg!(META)};
        unsafe{dbg!(&META.as_mut().unwrap().global)};
    }
}

impl Drop for Meta {
    fn drop(&mut self) {
        println!("Dropping the meta..");
    }
}

macro_rules! stack_push {
    ($stack:ident,$dtype:ty,$times:expr) => {
        for _time in 0..$times {
            $stack.0.push(UnsafeCell::from(<$dtype>::default()));
        }
    };
}

#[derive(Debug)]
pub struct Stack(Vec<UnsafeCell<StkElem>>);

impl Stack {
    /// brief: alloc a new stack with capacity and length
    /// note that capacity >= length
    #[inline]
    fn new(capacity: usize, length: usize) -> Option<Stack> {
        // return error if length is greater than capacity
        if length > capacity {
            return None;
        }

        let mut stk = Stack {
            0: Vec::with_capacity(capacity),
        };

        stack_push!(stk, StkElem, length);
        return Some(stk);
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn get_mut_elem(&self, index: usize) -> Option<&mut StkElem> {
        if let Some(stk) = self.0.get(index) {
            Some(unsafe { &mut *(stk.get()) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn get_ref_elem(&self, index: usize) -> Option<&StkElem> {
        if let Some(stk) = self.0.get(index) {
            Some(unsafe { &*(stk.get()) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn get_ptr(&self, index: usize) -> Option<*mut StkElem> {
        if let Some(stk) = self.0.get(index) {
            Some(stk.get())
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn get_elem(&self, index: usize) -> Option<StkElem> {
        if let Some(stk) = self.0.get(index) {
            Some(unsafe { *(stk.get()) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn swap_elem(&self, index: usize, new_stkelem: &mut StkElem) -> Result<ErrCode, ErrCode> {
        if let Some(s) = self.0.get(index) {
            let stk = s.get();
            if !stk.is_null() {
                swap(unsafe { &mut *stk }, new_stkelem);
            } else {
                return Err(ErrCode::NullPointer);
            }
            return Ok(ErrCode::Fine);
        } else {
            return Err(ErrCode::NoneObject);
        }
    }

    fn increase(&mut self, need: usize) -> Result<usize, ErrCode> {
        // the space that has been allocated
        let old_alloc = self.0.len();

        if old_alloc > LUA_MAX_STACK as usize {
            return Err(ErrCode::OverFlow);
        }
        // a branch that program will not step in for sure

        let mut to_add = old_alloc;
        let to_add2 = need + LUA_EXTRASPACE;

        // apply the larger one
        if to_add < to_add2 {
            to_add = to_add2;
        }

        if old_alloc + to_add > LUA_MAX_STACK as usize {
            return Err(ErrCode::OverFlow);
        }

        // capacity >= length
        stack_push!(self, StkElem, to_add);

        return Ok(to_add);
    }

    #[allow(dead_code)]
    fn decrease(&mut self, _starting_pos: usize) {}
}

impl Drop for Stack {
    fn drop(&mut self) {
        self.0.clear();
    }
}

#[derive(Debug)]
pub struct CallInfoVec(Vec<UnsafeCell<CallInfo>>);

impl CallInfoVec {
    fn new(capacity: usize, length: usize) -> Option<CallInfoVec> {
        // return error if length is greater than capacity
        if length > capacity {
            return None;
        }

        let mut civ = CallInfoVec {
            0: Vec::with_capacity(capacity),
        };
        //unsafe { civ.0.set_len(length) };
        stack_push!(civ, CallInfo, length);
        dbg!(civ.0.len());
        Some(civ)
    }

    pub fn swap_elem(&self, index: usize, new_ci: &mut CallInfo) -> Result<ErrCode, ErrCode> {
        if let Some(c) = self.0.get(index) {
            let ci = c.get();
            if !ci.is_null() {
                swap(unsafe { &mut *ci }, new_ci);
            } else {
                return Err(ErrCode::NullPointer);
            }
            return Ok(ErrCode::Fine);
        } else {
            return Err(ErrCode::NoneObject);
        }
    }

    fn increase(&mut self, civ_top_index: usize, need: usize) -> Result<ErrCode, ErrCode> {
        // the space that has been allocated
        let old_alloc = self.0.len();

        if old_alloc > LUA_MAX_CALLS as usize {
            return Err(ErrCode::OverFlow);
        }
        // will never happen

        if civ_top_index * 2 > old_alloc || civ_top_index + need > old_alloc {
            let mut to_add = old_alloc;
            let to_add2 = need;

            // apply the larger one
            if to_add < to_add2 {
                to_add = to_add2;
            }

            if old_alloc + to_add > LUA_MAX_STACK as usize {
                return Err(ErrCode::OverFlow);
            }
            // capacity >= length
            stack_push!(self, CallInfo, to_add);
            return Ok(ErrCode::Fine);
        }
        // need add space
        else {
            return Ok(ErrCode::Fine);
        } // it is not necessary to add
    }

    #[allow(dead_code)]
    fn decrease(&mut self, _starting_pos: usize) {}

    #[inline(always)]
    #[allow(dead_code)]
    pub fn get_ref_elem(&self, index: usize) -> Option<&CallInfo> {
        if let Some(ci) = self.0.get(index) {
            Some(unsafe { &*(ci.get()) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn get_mut_elem(&self, index: usize) -> Option<&mut CallInfo> {
        if let Some(ci) = self.0.get(index) {
            Some(unsafe { &mut *(ci.get()) })
        } else {
            None
        }
    }
}

impl Drop for CallInfoVec {
    fn drop(&mut self) {
        self.0.clear();
    }
}

const LUA_EXTRASPACE: usize = size_of::<*mut ()>();

#[derive(Default,Debug)]
#[allow(dead_code)]
pub struct LuaState {
    stack: Option<NonNull<Stack>>, // a pointer to the stack

    stack_last_index: usize,

    stack_top_index: usize,

    stack_size: usize,

    next: Option<NonNull<LuaState>>,     // default value: None
    previous: Option<NonNull<LuaState>>, // default value: None

    civ: Option<NonNull<CallInfoVec>>,
    ncalls: usize,
    //cci_index: usize,
    global: Option<NonNull<GlobalState>>,
    status: LuaStateStatus,
}

#[derive(Default,Debug)]
#[allow(dead_code)]
pub struct CallInfo {
    stack: Option<NonNull<Stack>>,
    stack_func_index: usize,
    stack_top_index: usize,
    nresult: isize,
    callstatus: LuaCallInfoStatus,
}

impl CallInfo {
    fn new(
        stk: Option<NonNull<Stack>>,
        stack_func_index: usize,
        stack_top_index: usize,
        nres: isize,
        status: LuaCallInfoStatus,
    ) -> Self {
        Self {
            stack: stk,
            stack_func_index,
            stack_top_index,
            nresult: nres,
            callstatus: status,
        }
    }

    fn ci_check(&self, size: usize) -> bool {
        if self.stack_func_index + size >= self.stack_top_index {
            false
        } else {
            true
        }
    }
}

impl LuaState {
    pub fn set_status(&mut self, status: LuaStateStatus) {
        self.status = status;
    }

    pub fn get_status(&self) -> LuaStateStatus {
        self.status
    }

    pub fn get_top_index(&self) -> usize {
        self.stack_top_index
    }

    pub fn change_ncalls(&mut self, step: usize, direction: bool) {
        self.ncalls = {
            if direction {
                self.ncalls + step
            } else {
                self.ncalls - step
            }
        }
    }

    pub fn write_ci_status(&self, ci_index: usize, status: LuaCallInfoStatus) {
        let cci: &mut CallInfo = ptr_get!(ptr_get!(self, civ).ok().unwrap().get_mut_elem(ci_index))
            .ok()
            .unwrap();
        cci.callstatus = status;
    }

    pub fn get_ci_status(&self, ci_index: usize) -> LuaCallInfoStatus {
        let cci = ptr_get!(ptr_get!(self, civ).ok().unwrap().get_ref_elem(ci_index))
            .ok()
            .unwrap();
        cci.callstatus
    }

    pub fn get_stack_mut_ref(&self) -> Option<&mut Stack> {
        if let Some(stack) = self.stack {
            let ptr = stack.as_ptr();
            if !ptr.is_null() {
                Some(unsafe { &mut *ptr })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_civ_mut_ref(&self) -> Option<&mut CallInfoVec> {
        if let Some(civ) = self.civ {
            let ptr = civ.as_ptr();
            if !ptr.is_null() {
                Some(unsafe { &mut *ptr })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn stack_init(&mut self) -> Result<ErrCode, ErrCode> {
        // initialize the stack, drop the memory manually

        let stk_opt = Stack::new(LUA_MAX_STACK as usize, LUA_STACK_SIZE as usize);
        // None type will return only if length size is greater that capacity

        if let Some(stack) = stk_opt {
            self.stack = Some(ptr_init!(Box::leak(Box::new(stack))));
            // static lifetime

            // set the current size of the stack
            self.stack_size = LUA_STACK_SIZE as usize;
            self.stack_last_index = 0usize + (LUA_STACK_SIZE - LUA_EXTRA_STACK) as usize;
            self.stack_top_index = 0;
            // pos 0 is assumed to take

            return Ok(ErrCode::Fine);
        } else {
            return Err(ErrCode::OverFlow);
        }
    }

    pub fn stack_check(&mut self, need: usize) {
        if self.stack_top_index + need > self.stack_last_index {
            self.stack_increase(need);
        }
    }

    /// true: legal
    /// false: illegal
    pub fn calls_check(&self) -> bool {
        !(self.ncalls >= LUA_MAX_CALLS)
    }

    fn stack_increase(&mut self, size: usize) {
        let size_add = ptr_get!(self, stack)
            .ok()
            .unwrap()
            .increase(size)
            .ok()
            .unwrap();
        self.stack_size += size_add;
        self.stack_last_index = self.stack_size - LUA_EXTRA_STACK as usize;
    }

    pub fn stack_shrink(&mut self, ci_index: usize) {
        let cci = ptr_get!(ptr_get!(self, civ).ok().unwrap().get_mut_elem(ci_index))
            .ok()
            .unwrap();
        let func_index = cci.stack_func_index;
        ptr_get!(self, stack).ok().unwrap().decrease(func_index);
        //stack_last_index:
        //stack_size

        cci.stack_top_index = ILLEGAL_INDEX;
        cci.stack_func_index = ILLEGAL_INDEX;
    }

    fn stack_clear(&mut self) {
        drop(self.stack.take());
        self.stack = None;
        self.stack_size = 0;
        self.stack_top_index = ILLEGAL_INDEX;
        self.stack_last_index = ILLEGAL_INDEX;
    }

    #[allow(dead_code)]
    pub fn civ_init(&mut self) -> Result<ErrCode, ErrCode> {
        let civ_opt = CallInfoVec::new(LUA_MAX_CALLS, LUA_CI_LEN);
        // None type will return only if length size is greater that capacity
        
        if let Some(civ) = civ_opt {
            let civ_box = Box::new(civ);
            // static lifetime
            dbg!(&civ_box.0.len());
            // let mut cci = CallInfo::new(
            //     self.stack,
            //     0,
            //     LUA_MIN_STACK as usize,
            //     Default::default(),
            //     LuaCallInfoStatus::CallOk,
            // ); // act as the main function

            // let _ = civ_box.swap_elem(0, &mut cci);

            self.civ = Some(ptr_init!(Box::leak(civ_box)));
            // static lifetime

            self.ncalls = 0;

            return Ok(ErrCode::Fine);
        } else {
            return Err(ErrCode::OverFlow);
        }
    }

    #[allow(dead_code)]
    pub fn add_next_ci(&mut self, func_index: usize, nresult: isize) -> usize {
        // try to increase the civ
        let civ_ptr = ptr_get!(self, civ).ok().unwrap();

        let _ = civ_ptr.increase(self.ncalls, 1).ok().unwrap();

        let mut ci = CallInfo::new(
            self.stack,
            func_index,
            self.stack_top_index + LUA_MIN_STACK as usize,
            nresult,
            LuaCallInfoStatus::CallOk,
        );

        let _ = civ_ptr.swap_elem(self.ncalls, &mut ci).ok().unwrap();

        self.ncalls += 1;
        return self.ncalls - 1;
    }

    pub fn cci_check(&self, index: usize, size: usize) -> bool {
        ptr_get!(ptr_get!(self, civ).ok().unwrap().get_ref_elem(index))
            .ok()
            .unwrap()
            .ci_check(size)
    }

    pub fn civ_shrink(&mut self, ci_index: usize) {
        // without any cleaning
        ptr_get!(self, civ).ok().unwrap().decrease(ci_index);
    }

    fn callvec_clear(&mut self) {
        drop(self.civ.take());
        //self.cci_index = ILLEGAL_INDEX;

        self.ncalls = 0; // no space
    }

    pub fn mainthread_new(ud: *const ()) -> &'static mut LuaState {
        // initialize meta, with default value
        Meta::new();

        // global state accepts userdata
        get_global_state!().userdata = Some(ptr_init!(unsafe { &*ud }));
        
        //get_main_state!() = Default::default();

        // link the state with global state
        get_main_state!().global = Some(ptr_init!(ptr_get!(get_global_state!(), NonNull)));

        // link the global state with the state
        get_global_state!().mainthread = Some(ptr_init!(ptr_get!(get_main_state!(), NonNull)));

        // stack initialize
        let _ = get_main_state!().stack_init().ok().unwrap();

        // civ initialize
        let _ = get_main_state!().civ_init().ok().unwrap();

        dbg!(&get_global_state!());
        dbg!(&get_main_state!());
        dbg!(&get_main_state!().civ);
        &mut get_main_state!()
    }

    #[inline(always)]
    pub fn move_top_to(&mut self, index: usize) {
        self.stack_top_index = index;
    }

    #[inline(always)]
    pub fn move_top(&mut self, step: usize, direction: bool) {
        dbg!(self.stack_top_index);
        assert!(self.stack_last_index>=self.stack_top_index);
        if direction {
            self.stack_top_index += step;
        } else {
            self.stack_top_index -= step;
        }
        dbg!(self.stack_top_index);
    }

    #[inline(always)]
    fn increase_top(&mut self) {
        self.move_top(1, true);
    }

    pub fn push_integer(&mut self, integer: INT) {
        let mut elem = StkElem::new_integer(integer);

        //unsafe{dbg!("before: ",&elem.get_value().val_int)};
        //dbg!("before: ",&elem.get_type());
        let _ = ptr_get!(self, stack)
            .ok()
            .unwrap()
            .swap_elem(self.stack_top_index, &mut elem)
            .ok()
            .unwrap();
        dbg!(self.stack_top_index);
        //dbg!("after push",unsafe{self.stack.unwrap().as_ref().0.get(self.stack_top_index).unwrap().get().as_ref().unwrap().get_value().val_int});
        //dbg!("after push",unsafe{self.stack.unwrap().as_ref().0.get(self.stack_top_index).unwrap().get().as_ref().unwrap().get_type()});
        self.increase_top();
        dbg!(self.stack_top_index);
    }

    pub fn push_float(&mut self, number: FLT) {
        let mut elem = StkElem::new_float(number);
        let _ = ptr_get!(self, stack)
            .ok()
            .unwrap()
            .swap_elem(self.stack_top_index, &mut elem)
            .ok()
            .unwrap();
        self.increase_top();
    }

    pub fn push_bool(&mut self, boolean: bool) {
        let mut elem = StkElem::new_bool(boolean);
        let _ = ptr_get!(self, stack)
            .ok()
            .unwrap()
            .swap_elem(self.stack_top_index, &mut elem)
            .ok()
            .unwrap();
        self.increase_top();
    }

    pub fn push_nil(&mut self) {
        let mut elem = StkElem::new_nil();
        let _ = ptr_get!(self, stack)
            .ok()
            .unwrap()
            .swap_elem(self.stack_top_index, &mut elem)
            .ok()
            .unwrap();
        self.increase_top();
    }

    pub fn push_ud(&mut self, ud: Option<*mut ()>) {
        if let Some(ud_) = ud {
            let mut elem = StkElem::new_ud(ud_);
            let _ = ptr_get!(self, stack)
                .ok()
                .unwrap()
                .swap_elem(self.stack_top_index, &mut elem)
                .ok()
                .unwrap();
        } else {
            let mut elem = StkElem::new_ud(null_mut());
            let _ = ptr_get!(self, stack)
                .ok()
                .unwrap()
                .swap_elem(self.stack_top_index, &mut elem)
                .ok()
                .unwrap();
        }
        self.increase_top();
    }

    pub fn push_rfunc(&mut self, rfunc: &RFUNC) {
        //let mut x=rfunc;
        //x(&mut get_main_state!());
        let mut elem = StkElem::new_rfunc(rfunc);
        //let x=unsafe{elem.get_value().val_rfunc.into_inner().unwrap().as_ref()};
        //x(&mut get_main_state!());
        //dbg!("push_rfunc",unsafe{elem.get_value().val_rfunc.into_inner()}.unwrap().as_ptr() as *const());
        let _ = ptr_get!(self, stack)
            .ok()
            .unwrap()
            .swap_elem(self.stack_top_index, &mut elem)
            .ok()
            .unwrap();
        self.increase_top();
    }

    pub fn push_obj(&mut self, obj: StkElem) {
        let mut elem = StkElem::new_obj(obj);
        let _ = ptr_get!(self, stack)
            .ok()
            .unwrap()
            .swap_elem(self.stack_top_index, &mut elem)
            .ok()
            .unwrap();
        self.increase_top();
    }

    pub fn push_errcode(&mut self, statecode: INT) {
        self.push_integer(statecode as INT);
    }

    pub fn pop_stack(&mut self) -> StkElem {
        dbg!(self.stack_top_index);
        let mut elem = StkElem::default();
        let _ = ptr_get!(self, stack)
            .ok()
            .unwrap()
            .swap_elem(self.stack_top_index - 1, &mut elem)
            .ok()
            .unwrap();

        self.move_top(1, false);
        elem
    }

    pub fn pop_integer(&mut self) -> INT {
        let elem = self.pop_stack();
        if elem.get_type() != TNumber::NumInt as u8 {
            panic!("FATAL ERROR: MISMATCH TYPE");
        } else {
            unsafe { elem.get_value().val_int }.into_inner().unwrap()
        }
    }

    pub fn pop_float(&mut self) -> FLT {
        let elem = self.pop_stack();
        if elem.get_type() != TNumber::NumFlt as u8 {
            panic!("FATAL ERROR: MISMATCH TYPE");
        } else {
            unsafe { elem.get_value().val_num }.into_inner().unwrap()
        }
    }

    pub fn pop_bool(&mut self) -> bool {
        let elem = self.pop_stack();
        if elem.get_type() != TObject::TBoolean as u8 {
            panic!("FATAL ERROR: MISMATCH TYPE");
        } else {
            unsafe { elem.get_value().val_bl }.into_inner().unwrap()
        }
    }

    pub fn pop_nil(&mut self) {
        let elem = self.pop_stack();
        if elem.get_type() != TObject::TNil as u8 {
            panic!("FATAL ERROR: MISMATCH TYPE");
        }
    }

    pub fn pop_ud(&mut self) -> *const () {
        let elem = self.pop_stack();
        if elem.get_type() != TObject::TLightUserData as u8 {
            panic!("FATAL ERROR: MISMATCH TYPE");
        } else {
            unsafe { elem.get_value().val_ud }.into_inner().unwrap()
        }
    }
}

impl Drop for LuaState {
    fn drop(&mut self) {
        self.stack_clear();
        self.callvec_clear();
    }
}

// mod test{
//     use crate::common::state::statedef::LuaState;

    
    
    

//     #[test]
    
//     fn test1(){
//         let state=LuaState::mainthread_new(null_mut());
//         state.push_integer(3);
//         dbg!("starting to pop");
//         let x=state.pop_integer();
//         println!("{}",x);
//     }
// }
