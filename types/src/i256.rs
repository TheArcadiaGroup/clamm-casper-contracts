use crate::i128::{self as i128p, I128};
use alloc::{string::ToString, vec, vec::Vec};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, U256,
};
use core::ops;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct I256(pub ethnum::i256);

impl FromBytes for I256 {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (words, bytes) = Vec::<I128>::from_bytes(bytes)?;
        let word0 = words[0].0;
        let word1 = words[1].0;
        let v: ethnum::prelude::I256 = ethnum::i256::from_words(word0, word1);
        Ok((I256(v), bytes))
    }
}

impl Default for I256 {
    fn default() -> Self {
        Self(0.into())
    }
}

impl ToBytes for I256 {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let (word0, word1) = self.0.into_words();
        let word0 = i128p::I128(word0);
        let word1 = i128p::I128(word1);
        vec![word0, word1].to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.to_le_bytes().len()
    }
}

impl CLTyped for I256 {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::Any
    }
}

impl From<U256> for I256 {
    fn from(v: U256) -> Self {
        I256(ethnum::i256::from_str_radix(&v.to_string(), 10).unwrap())
    }
}

impl From<I256> for U256 {
    fn from(v: I256) -> Self {
        U256::from_dec_str(&v.0.to_string()).unwrap()
    }
}

impl From<ethnum::i256> for I256 {
    fn from(v: ethnum::i256) -> Self {
        I256(v)
    }
}

impl From<I256> for ethnum::i256 {
    fn from(v: I256) -> Self {
        v.0
    }
}

impl ops::Add<I256> for I256 {
    type Output = I256;
    fn add(self, _rhs: I256) -> Self::Output {
        I256(self.0 + _rhs.0)
    }
}

impl ops::Sub<I256> for I256 {
    type Output = I256;
    fn sub(self, _rhs: I256) -> Self::Output {
        I256(self.0 - _rhs.0)
    }
}

impl ops::Neg for I256 {
    type Output = I256;
    fn neg(self) -> Self::Output {
        I256(-self.0)
    }
}

impl ops::Div<I256> for I256 {
    type Output = I256;
    fn div(self, _rhs: I256) -> Self::Output {
        I256(self.0 / _rhs.0)
    }
}

impl ops::Mul<I256> for I256 {
    type Output = I256;
    fn mul(self, _rhs: I256) -> Self::Output {
        I256(self.0 * _rhs.0)
    }
}

impl ops::BitOr<I256> for I256 {
    type Output = I256;
    fn bitor(self, rhs: I256) -> Self::Output {
        self.0.bitor(rhs.0).into()
    }
}

impl ops::Shl<I256> for I256 {
    type Output = I256;
    fn shl(self, rhs: I256) -> Self::Output {
        I256(self.0.shl(rhs.0))
    }
}

impl ops::Shr<I256> for I256 {
    type Output = I256;
    fn shr(self, rhs: I256) -> Self::Output {
        I256(self.0.shr(rhs.0))
    }
}

impl From<&str> for I256 {
    fn from(value: &str) -> Self {
        I256(ethnum::i256::from_str_radix(value, 10).unwrap())
    }
}

impl PartialOrd for I256 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// impl AssertParamIsEq for I256 {

// }

macro_rules! impl_from {
    ($($t:ty),* $(,)?) => {$(
        impl From<$t> for I256 {
            #[inline]
            fn from(value: $t) -> Self {
                Self(ethnum::i256::from(value))
            }
        }
    )*};
}

impl_from! {
    bool,
    i8, i16, i32, i64, i128,
    u8, u16, u32, u64, u128,
}
