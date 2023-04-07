pub trait ObjectTrait {
    type Item;

    fn new(val: Option<Self::Item>) -> Self;

    fn is_none(&self) -> bool;

    fn is_some(&self) -> bool;

    fn reveal_type(&self) -> u8;

    fn set_value(&mut self, val: Option<Self::Item>);

    fn into_inner(&mut self) -> Option<Self::Item>;
}
