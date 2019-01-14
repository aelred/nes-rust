use enum_primitive_derive::Primitive;
use crate::Memory;
use crate::Address;

#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum Mapper {
    NROM = 0,
    Namco129 = 19
}
