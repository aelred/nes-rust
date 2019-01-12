use crate::SerializeBytes;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Address(u16);

impl Address {
    pub const fn new(value: u16) -> Self {
        Address(value)
    }

    pub const fn zero_page(value: u8) -> Self {
        Address::new(value as u16)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn split(self) -> (u8, u8) {
        ((self.0 >> 8) as u8, self.0 as u8)
    }
}

impl SerializeBytes for Address {
    const SIZE: u8 = 2;

    type Iter = std::vec::IntoIter<u8>;

    fn serialize(self) -> Self::Iter {
        let (higher, lower) = self.split();
        vec![lower, higher].into_iter()
    }

    fn deserialize(mut source: impl Iterator<Item = u8>) -> Self {
        let lower = source.next().unwrap();
        let higher = source.next().unwrap();
        Address((u16::from(higher) << 8) + u16::from(lower))
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Address::new(value)
    }
}

impl AddAssign<i8> for Address {
    fn add_assign(&mut self, rhs: i8) {
        self.0 = self.0.wrapping_add(rhs as u16);
    }
}

impl AddAssign<u8> for Address {
    fn add_assign(&mut self, rhs: u8) {
        self.0 = self.0.wrapping_add(u16::from(rhs));
    }
}

impl Add<u8> for Address {
    type Output = Address;

    fn add(self, rhs: u8) -> <Self as Add<u8>>::Output {
        Address(self.0.wrapping_add(u16::from(rhs)))
    }
}

impl Sub<u8> for Address {
    type Output = Address;

    fn sub(self, rhs: u8) -> <Self as Sub<u8>>::Output {
        Address(self.0.wrapping_sub(u16::from(rhs)))
    }
}

impl AddAssign<u16> for Address {
    fn add_assign(&mut self, rhs: u16) {
        self.0 = self.0.wrapping_add(rhs);
    }
}

impl Add<u16> for Address {
    type Output = Address;

    fn add(self, rhs: u16) -> <Self as Add<u16>>::Output {
        Address(self.0.wrapping_add(rhs))
    }
}
