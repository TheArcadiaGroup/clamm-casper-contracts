use core::ops::{BitAnd, Shr};

use casper_types::U256;
use common::error::{require, Error};

pub fn most_significant_bit(x: U256) -> u8 {
    require(x > U256::zero(), Error::ErrMoreThan);
    most_significant_bit_internal(x)
}

pub fn most_significant_bit_internal(x: U256) -> u8 {
    let mut x = x;
    let mut r = 0_u8;
    if x > u128::MAX.into() {
        x = x.shr(128);
        r += 128;
    }
    if x > u64::MAX.into() {
        x = x.shr(64);
        r += 64;
    }
    if x > u32::MAX.into() {
        x = x.shr(32);
        r += 32;
    }
    if x > u16::MAX.into() {
        x = x.shr(16);
        r += 16;
    }
    if x > u8::MAX.into() {
        x = x.shr(8);
        r += 8;
    }
    if x > 15_u128.into() {
        x = x.shr(4);
        r += 4;
    }
    if x > 3_u128.into() {
        x = x.shr(2);
        r += 2;
    }
    if x > 1_u128.into() {
        r += 1;
    }
    r
}

pub fn least_significant_bit(x: U256) -> u8 {
    require(x > U256::zero(), Error::ErrMoreThan);
    least_significant_bit_internal(x)
}

pub fn least_significant_bit_internal(x: U256) -> u8 {
    let mut x = x;
    let mut r = 255;
    if x.bitand(u128::MAX.into()) > 0_u128.into() {
        r -= 128;
    } else {
        x = x.shr(128);
    }
    if x.bitand(u64::MAX.into()) > 0_u128.into() {
        r -= 64;
    } else {
        x = x.shr(64);
    }
    if x.bitand(u32::MAX.into()) > 0_u128.into() {
        r -= 32;
    } else {
        x = x.shr(32);
    }
    if x.bitand(u16::MAX.into()) > 0_u128.into() {
        r -= 16;
    } else {
        x = x.shr(16);
    }
    if x.bitand(u8::MAX.into()) > 0_u128.into() {
        r -= 8;
    } else {
        x = x.shr(8);
    }
    if x.bitand(15_u128.into()) > 0_u128.into() {
        r -= 4;
    } else {
        x = x.shr(4);
    }
    if x.bitand(3_u128.into()) > 0_u128.into() {
        r -= 2;
    } else {
        x = x.shr(2);
    }
    if x.bitand(1_u128.into()) > 0_u128.into() {
        r -= 1;
    }
    r
}
