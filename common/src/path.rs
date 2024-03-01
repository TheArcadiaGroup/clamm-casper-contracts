use casper_types::{
    bytesrepr::Bytes,
    Key,
};

use crate::utils::{set_size_32, set_size_4};

pub const ADDR_SIZE: usize = 32;
pub const FEE_SIZE: usize = 4;
// The offset of a single token address and pool fee
pub const NEXT_OFFSET: usize = ADDR_SIZE + FEE_SIZE;
// The offset of an encoded pool key
pub const POP_OFFSET: usize = NEXT_OFFSET + ADDR_SIZE;
pub const MULTIPLE_POOLS_MIN_LENGTH: usize = POP_OFFSET + NEXT_OFFSET;

pub fn has_multiple_pools(path: &[u8]) -> bool {
    path.len() >= MULTIPLE_POOLS_MIN_LENGTH
}

pub fn num_pools(path: &[u8]) -> usize {
    (path.len() - ADDR_SIZE) / NEXT_OFFSET
}

pub fn decode_first_pool(path: &[u8]) -> (Key, Key, u32) {
    let token_a = &path[0..ADDR_SIZE];
    let token_a = Key::Hash(set_size_32(token_a));

    let fee = &path[ADDR_SIZE..NEXT_OFFSET];
    let fee = u32::from_le_bytes(set_size_4(fee));

    let token_b = &path[NEXT_OFFSET..NEXT_OFFSET + ADDR_SIZE];
    let token_b = Key::Hash(set_size_32(token_b));

    (token_a, token_b, fee)
}

pub fn get_first_pool(path: &[u8]) -> Bytes {
    path[0..POP_OFFSET].into()
}

pub fn skip_token(path: &[u8]) -> Bytes {
    path[NEXT_OFFSET..path.len()].into()
}
