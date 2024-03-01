use alloc::{string::ToString, vec::Vec};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, U128,
};
use core::ops;

#[derive(Debug, Clone, Copy, Default)]
pub struct I128(pub i128);

impl FromBytes for I128 {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (signed, bytes) = u8::from_bytes(bytes)?;
        let (val, bytes) = U128::from_bytes(bytes)?;
        let val = val.as_u128();
        let val = if signed == 0 {
            -(val as i128)
        } else {
            val as i128
        };
        Ok((I128(val), bytes))
    }
}

impl ToBytes for I128 {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let signed: u8 = if self.0 < 0 { 0 } else { 1 };
        let val = self.0.unsigned_abs();
        let mut vec = Vec::with_capacity(self.serialized_length());
        vec.append(&mut signed.to_bytes()?);
        vec.append(&mut U128::from(val).to_bytes()?);
        Ok(vec)
    }

    fn serialized_length(&self) -> usize {
        let val = self.0.unsigned_abs();
        1 + U128::from(val).serialized_length()
    }
}

impl CLTyped for I128 {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::Any
    }
}

impl From<u128> for I128 {
    fn from(v: u128) -> Self {
        I128(v.to_string().parse().unwrap())
    }
}

impl From<i128> for I128 {
    fn from(v: i128) -> Self {
        I128(v)
    }
}

impl From<I128> for i128 {
    fn from(v: I128) -> Self {
        v.0
    }
}

impl ops::Add<I128> for I128 {
    type Output = I128;
    fn add(self, _rhs: I128) -> Self::Output {
        I128(self.0 + _rhs.0)
    }
}

impl ops::Sub<I128> for I128 {
    type Output = I128;
    fn sub(self, _rhs: I128) -> Self::Output {
        I128(self.0 - _rhs.0)
    }
}

impl ops::Neg for I128 {
    type Output = I128;
    fn neg(self) -> Self::Output {
        I128(-self.0)
    }
}

impl ops::Div<I128> for I128 {
    type Output = I128;
    fn div(self, _rhs: I128) -> Self::Output {
        I128(self.0 / _rhs.0)
    }
}

impl ops::Mul<I128> for I128 {
    type Output = I128;
    fn mul(self, _rhs: I128) -> Self::Output {
        I128(self.0 * _rhs.0)
    }
}

impl ops::BitOr<I128> for I128 {
    type Output = I128;
    fn bitor(self, rhs: I128) -> Self::Output {
        self.0.bitor(rhs.0).into()
    }
}

impl ops::Shl<I128> for I128 {
    type Output = I128;
    fn shl(self, rhs: I128) -> Self::Output {
        I128(self.0.shl(rhs.0))
    }
}

impl ops::Shr<I128> for I128 {
    type Output = I128;
    fn shr(self, rhs: I128) -> Self::Output {
        I128(self.0.shr(rhs.0))
    }
}

impl From<&str> for I128 {
    fn from(value: &str) -> Self {
        I128(value.parse().unwrap())
    }
}

macro_rules! impl_from {
    ($($t:ty),* $(,)?) => {$(
        impl From<$t> for I128 {
            #[inline]
            fn from(value: $t) -> Self {
                Self(i128::from(value))
            }
        }
    )*};
}

impl_from! {
    bool,
    i8, i16, i32, i64,
    u8, u16, u32, u64,
}
