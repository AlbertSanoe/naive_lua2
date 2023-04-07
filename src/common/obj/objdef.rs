use std::ptr::NonNull;

use super::{
    objtrait::ObjectTrait,
    objtype::{Bool, DataType, Integer, Nil, Number, RFunction, UserData, FLT, INT, RFUNC},
};

#[derive(Debug)]
pub enum TObject {
    TNumber = 1,
    TLightUserData = 2,
    TBoolean = 3,
    TString = 4,
    TNil = 5,
    TTable = 6,
    TFunction = 7,
    TThread = 8,
    TNone = 9,
}

pub const BASIC_TYPE_BIT: usize = 4;

impl TObject {
    pub fn is_function(label: u8) -> bool {
        if label & 7u8 == 7 {
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub enum TNumber {
    NumInt = (TObject::TNumber as isize | (0 << 4)), //1
    NumFlt = (TObject::TNumber as isize | (1 << 4)), //17
}

#[derive(Debug)]
pub enum TFuction {
    TLCL = (TObject::TFunction as isize | (0 << 4)), //7
    TLRF = (TObject::TFunction as isize | (1 << 4)), //23 type: light rust function
    TCCL = (TObject::TFunction as isize | (2 << 4)), //39
}

#[derive(Debug)]
pub enum TString {
    LngStr = (TObject::TString as isize | (0 << 4)), //4
    ShrStr = (TObject::TString as isize | (1 << 4)), //20
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct LuaTObject {
    value: DataType,
    val_type: u8,
}

pub type TObj = LuaTObject;

impl Default for LuaTObject {
    fn default() -> Self {
        Self {
            value: Default::default(),
            val_type: Default::default(),
        }
    }
}

impl LuaTObject {
    #[inline(always)]
    pub fn get_type(&self) -> u8 {
        self.val_type
    }

    #[inline(always)]
    pub fn get_value(&self) -> DataType {
        self.value
    }

    pub fn new_integer(integer: INT) -> Self {
        let mut obj = LuaTObject::default();
        obj.set_integer(integer);
        obj
    }

    #[inline(always)]
    pub fn set_integer(&mut self, integer: INT) {
        self.value.val_int = Integer::new(Some(integer));
        self.val_type = TNumber::NumInt as u8;
    }

    pub fn new_float(number: FLT) -> Self {
        let mut obj = LuaTObject::default();
        obj.set_float(number);
        obj
    }

    #[inline(always)]
    pub fn set_float(&mut self, number: FLT) {
        self.value.val_num = Number::new(Some(number));
        self.val_type = TNumber::NumFlt as u8;
    }

    pub fn new_bool(boolean: bool) -> Self {
        let mut obj = LuaTObject::default();
        obj.set_bool(boolean);
        obj
    }

    #[inline(always)]
    pub fn set_bool(&mut self, boolean: bool) {
        self.value.val_bl = Bool::new(Some(boolean));
        self.val_type = TObject::TBoolean as u8;
    }

    pub fn new_nil() -> Self {
        LuaTObject::default()
    }

    #[inline(always)]
    pub fn set_nil(&mut self) {
        self.value.val_nil = Nil::new(None);
        self.val_type = TObject::TNil as u8;
    }

    pub fn new_ud(ud: *mut ()) -> Self {
        let mut obj = LuaTObject::default();
        obj.set_ud(ud);
        obj
    }

    #[inline(always)]
    pub fn set_ud(&mut self, ud: *mut ()) {
        self.value.val_ud = UserData::new(Some(ud));
        self.val_type = TObject::TLightUserData as u8;
    }

    pub fn new_rfunc(rfunc: &RFUNC) -> Self {
        let mut obj = LuaTObject::default();
        obj.set_rfunc(rfunc);
        //dbg!(rfunc as *const());
        obj
    }

    pub fn set_rfunc(&mut self, rfunc: &RFUNC) {
        //let state=Machine::machine().get_state();
        //dbg!("set_rfunc");
        //rfunc(state);
        //self.value.val_rfunc =
        //let x=RFunction::new(Some((rfunc)));
        self.value.val_rfunc = RFunction::new(Some(NonNull::from(rfunc)));
        //dbg!("ddddddddddddddddddd",unsafe{self.value.val_rfunc.into_inner()}.unwrap().as_ptr() as *const());
        self.val_type = TFuction::TLRF as u8;
    }

    pub fn new_obj(obj: LuaTObject) -> Self {
        let mut _obj = LuaTObject::default();
        _obj.set_obj(obj);
        _obj
    }

    #[inline(always)]
    pub fn set_obj(&mut self, obj: LuaTObject) {
        self.val_type = obj.val_type;
        self.value = obj.value;
    }
}
