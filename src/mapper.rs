use enum_primitive_derive::Primitive;

#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum Mapper {
    NROM = 0,
    MMC1 = 1,
    Namco129 = 19,
}
