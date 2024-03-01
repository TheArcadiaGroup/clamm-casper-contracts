use core::{ops::Shl, str::FromStr};

use alloc::string::ToString;
use casper_types::U256;
use common::error::require;

use types::i128::I128;
use types::i256::I256;

pub fn to_i128(y: I256) -> I128 {
    let c: ethnum::i256 = y.into();
    c.as_i128().into()
}

pub fn to_i256(y: U256) -> I256 {
    let c = ethnum::i256::from_str(&y.to_string()).unwrap();
    c.into()
}

pub fn to_u160(y: U256) -> U256 {
    require(
        y <= U256::one().shl(160),
        common::error::Error::ConvertU160Overflow,
    );
    y
}
