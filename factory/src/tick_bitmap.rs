use core::ops::{BitAnd, BitXor, Shl, Shr};

use casper_types::U256;
use common::error::{require, Error};
use math::bitmath;

use crate::store::{read_tick_bitmap, save_tick_bitmap};

pub fn position(tick: i32) -> (i16, u8) {
    (tick.shr(8) as i16, (tick % 256) as u8)
}

pub fn flip_tick(tick: i32, tick_spacing: i32) {
    require(tick % tick_spacing == 0, Error::ErrFlipTick);
    let (word_pos, bit_pos) = position(tick / tick_spacing);
    let mask = U256::one().shl(bit_pos);
    let previous_mask = read_tick_bitmap(&word_pos.into());
    save_tick_bitmap(&word_pos.into(), &previous_mask.bitxor(mask));
}

pub fn tick_next_initialized_tick_within_one_word(
    tick: i32,
    tick_spacing: i32,
    lte: bool,
) -> (i32, bool) {
    let mut compressed = tick / tick_spacing;
    if tick < 0 && tick % tick_spacing != 0 {
        compressed -= 1;
    }

    if lte {
        let (word_pos, bit_pos) = position(compressed);
        let mask = U256::one().shl(bit_pos) - U256::one() + U256::one().shl(bit_pos);
        let masked = read_tick_bitmap(&word_pos.into()).bitand(mask);
        let initialized = masked != U256::zero();
        (
            if initialized {
                compressed
                    .overflowing_sub(
                        bit_pos
                            .overflowing_sub(bitmath::most_significant_bit(masked))
                            .0
                            .into(),
                    )
                    .0
                    * tick_spacing
            } else {
                compressed.overflowing_sub(bit_pos.into()).0 * tick_spacing
            },
            initialized,
        )
    } else {
        let (word_pos, bit_pos) = position(compressed + 1);
        let mask = U256::one().shl(bit_pos).overflowing_sub(U256::one()).0;
        let mask = mask.bitxor(U256::MAX);
        let masked = read_tick_bitmap(&word_pos.into()).bitand(mask);
        let initialized = masked != U256::zero();
        (
            if initialized {
                compressed
                    .overflowing_add(1)
                    .0
                    .overflowing_add(
                        bitmath::least_significant_bit(masked)
                            .overflowing_sub(bit_pos)
                            .0
                            .into(),
                    )
                    .0
                    * tick_spacing
            } else {
                compressed
                    .overflowing_add(1)
                    .0
                    .overflowing_add((255 - bit_pos).into())
                    .0
                    * tick_spacing
            },
            initialized,
        )
    }
}
