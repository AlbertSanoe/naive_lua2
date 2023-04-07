use std::ptr::NonNull;

use crate::common::state::statedef::LuaState;

use super::{
    objdef::{TNumber, TObject},
    objtrait::ObjectTrait,
};

pub type INT = i32; // integer
pub type FLT = f32; // float
pub type RFUNC = dyn Fn(&mut LuaState) -> usize;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub union DataType {
    pub val_int: Integer,
    pub val_ud: UserData,
    pub val_bl: Bool,
    pub val_nil: Nil,
    pub val_num: Number,
    pub val_rfunc: RFunction,
}

impl Default for DataType {
    fn default() -> Self {
        Self { val_nil: Nil() }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UserData(Option<*const ()>);

#[derive(Debug, Default, Clone, Copy)]
pub struct Bool(Option<bool>);

#[derive(Debug, Default, Clone, Copy)]
pub struct Integer(Option<INT>);

#[derive(Debug, Default, Clone, Copy)]
pub struct Number(Option<FLT>);

#[derive(Debug, Default, Clone, Copy)]
pub struct RFunction(Option<NonNull<RFUNC>>);

#[derive(Debug, Default, Clone, Copy)]
pub struct Nil();

impl ObjectTrait for UserData {
    type Item = *const ();

    fn new(mut val: Option<Self::Item>) -> Self {
        Self { 0: val.take() }
    }

    #[inline]
    fn is_none(&self) -> bool {
        self.0.is_none()
    }

    #[inline]
    fn is_some(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    fn reveal_type(&self) -> u8 {
        TObject::TLightUserData as u8
    }

    #[inline]
    fn set_value(&mut self, mut val: Option<Self::Item>) {
        self.0 = val.take();
    }

    #[inline]
    fn into_inner(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

impl ObjectTrait for Bool {
    type Item = bool;

    fn new(mut val: Option<Self::Item>) -> Self {
        Self { 0: val.take() }
    }

    #[inline]
    fn is_none(&self) -> bool {
        self.0.is_none()
    }

    #[inline]
    fn is_some(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    fn reveal_type(&self) -> u8 {
        TObject::TBoolean as u8
    }

    #[inline]
    fn set_value(&mut self, mut val: Option<Self::Item>) {
        self.0 = val.take();
    }

    #[inline]
    fn into_inner(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

impl ObjectTrait for Integer {
    type Item = INT;

    fn new(mut val: Option<Self::Item>) -> Self {
        Self { 0: val.take() }
    }

    #[inline]
    fn is_none(&self) -> bool {
        self.0.is_none()
    }

    #[inline]
    fn is_some(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    fn reveal_type(&self) -> u8 {
        TNumber::NumInt as u8
    }

    #[inline]
    fn set_value(&mut self, mut val: Option<Self::Item>) {
        self.0 = val.take();
    }

    fn into_inner(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

impl ObjectTrait for Number {
    type Item = FLT;

    fn new(mut val: Option<Self::Item>) -> Self {
        Self { 0: val.take() }
    }

    #[inline]
    fn is_none(&self) -> bool {
        self.0.is_none()
    }

    #[inline]
    fn is_some(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    fn reveal_type(&self) -> u8 {
        TNumber::NumFlt as u8
    }

    #[inline]
    fn set_value(&mut self, mut val: Option<Self::Item>) {
        self.0 = val.take();
    }

    fn into_inner(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

impl ObjectTrait for RFunction {
    type Item = NonNull<RFUNC>;

    fn new(mut val: Option<Self::Item>) -> Self {
        Self { 0: val.take() }
    }

    fn is_none(&self) -> bool {
        self.0.is_none()
    }

    fn is_some(&self) -> bool {
        self.0.is_some()
    }

    fn reveal_type(&self) -> u8 {
        TObject::TBoolean as u8
    }

    fn set_value(&mut self, mut val: Option<Self::Item>) {
        self.0 = val.take();
    }

    fn into_inner(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

impl ObjectTrait for Nil {
    type Item = ();

    fn new(_val: Option<Self::Item>) -> Self {
        Self {}
    }

    #[inline]
    fn is_none(&self) -> bool {
        true
    }

    #[inline]
    fn is_some(&self) -> bool {
        false
    }

    #[inline]
    fn reveal_type(&self) -> u8 {
        TObject::TNil as u8
    }

    #[inline]
    fn set_value(&mut self, _val: Option<Self::Item>) {}

    fn into_inner(&mut self) -> Option<Self::Item> {
        Some(())
    }
}
