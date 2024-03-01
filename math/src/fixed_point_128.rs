use casper_types::U256;

pub fn q128() -> U256 {
    U256::from_str_radix("100000000000000000000000000000000", 16).unwrap()
}
