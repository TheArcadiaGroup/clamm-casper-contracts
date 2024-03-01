use casper_types::U256;

pub const RESOLUTION: u8 = 96;

pub fn q96() -> U256 {
    U256::from_str_radix("1000000000000000000000000", 16).unwrap()
}
