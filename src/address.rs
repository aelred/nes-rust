use std::fmt;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Address(u16);

impl Address {
    pub const fn new(value: u16) -> Self {
        Address(value)
    }

    pub fn from_bytes(higher: u8, lower: u8) -> Self {
        Address(u16::from_le_bytes([lower, higher]))
    }

    pub fn bytes(self) -> u16 {
        self.0
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn higher(self) -> u8 {
        self.0.to_le_bytes()[1]
    }

    pub fn lower(self) -> u8 {
        self.0.to_le_bytes()[0]
    }

    pub fn incr_lower(self) -> Self {
        Address::from_bytes(self.higher(), self.lower().wrapping_add(1))
    }

    pub fn page_crossed(self, other: Address) -> bool {
        self.higher() != other.higher()
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Address({:#06x})", self.0)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#06x}", self.0)
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Address::new(value)
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

impl Sub<u16> for Address {
    type Output = Address;

    fn sub(self, rhs: u16) -> <Self as Sub<u16>>::Output {
        Address(self.0.wrapping_sub(rhs))
    }
}

impl Sub<Address> for Address {
    type Output = Address;

    fn sub(self, rhs: Address) -> <Self as Sub<Address>>::Output {
        self - rhs.0
    }
}
