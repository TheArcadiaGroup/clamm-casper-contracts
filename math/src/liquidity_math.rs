use alloc::format;
use common::{console, error::require};

pub fn add_delta(x: u128, y: i128) -> u128 {
    if y < 0 {
        let minus_y = y.unsigned_abs();
        console::log(&format!("add_delta {:?}, {:?}", x, minus_y));
        require(x >= minus_y, common::error::Error::ErrLS);
        x - minus_y
    } else {
        require(u128::MAX - x >= y as u128, common::error::Error::ErrLA);
        x + y as u128
    }
}
