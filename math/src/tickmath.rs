use core::ops::{BitOr, Shl, Shr};

use casper_types::U256;
use common::error::require;
use contract_utilities::helpers::{u256_to_u512, u512_to_u256};

use types::i256::I256;

pub const MIN_TICK: i32 = -887272;
pub const MAX_TICK: i32 = -MIN_TICK;

pub fn min_sqrt_ratio() -> U256 {
    U256::from(4295128739_u64)
}

pub fn max_sqrt_ratio() -> U256 {
    U256::from_dec_str("1461446703485210103287273052203988822378723970342").unwrap()
}

pub fn get_sqrt_ratio_at_tick(tick: i32) -> U256 {
    let abs_tick = tick.unsigned_abs() as u64;

    require(abs_tick <= MAX_TICK as u64, common::error::Error::ErrT);

    let mut ratio: U256 = if (abs_tick & 1u64) != 0 {
        U256::from(340265354078544963557816517032075149313_u128)
    } else {
        U256::from(u128::MAX) + U256::one()
    };
    if (abs_tick & 2u64) != 0 {
        ratio = (ratio * U256::from(340248342086729790484326174814286782778_u128)).shr(128);
    };
    if (abs_tick & 4u64) != 0 {
        ratio = (ratio * U256::from(340214320654664324051920982716015181260_u128)).shr(128);
    };
    if (abs_tick & 8u64) != 0 {
        ratio = (ratio * U256::from(340146287995602323631171512101879684304_u128)).shr(128);
    };
    if (abs_tick & 16u64) != 0 {
        ratio = (ratio * U256::from(340010263488231146823593991679159461444_u128)).shr(128);
    };
    if (abs_tick & 32u64) != 0 {
        ratio = (ratio * U256::from(339738377640345403697157401104375502016_u128)).shr(128);
    };
    if (abs_tick & 64u64) != 0 {
        ratio = (ratio * U256::from(339195258003219555707034227454543997025_u128)).shr(128);
    };
    if (abs_tick & 128u64) != 0 {
        ratio = (ratio * U256::from(338111622100601834656805679988414885971_u128)).shr(128);
    };
    if (abs_tick & 256u64) != 0 {
        ratio = (ratio * U256::from(335954724994790223023589805789778977700_u128)).shr(128);
    };
    if (abs_tick & 512u64) != 0 {
        ratio = (ratio * U256::from(331682121138379247127172139078559817300_u128)).shr(128);
    };
    if (abs_tick & 1024u64) != 0 {
        ratio = (ratio * U256::from(323299236684853023288211250268160618739_u128)).shr(128);
    };
    if (abs_tick & 2048u64) != 0 {
        ratio = (ratio * U256::from(307163716377032989948697243942600083929_u128)).shr(128);
    };
    if (abs_tick & 4096u64) != 0 {
        ratio = (ratio * U256::from(277268403626896220162999269216087595045_u128)).shr(128);
    };
    if (abs_tick & 8192u64) != 0 {
        ratio = (ratio * U256::from(225923453940442621947126027127485391333_u128)).shr(128);
    };
    if (abs_tick & 16384u64) != 0 {
        ratio = (ratio * U256::from(149997214084966997727330242082538205943_u128)).shr(128);
    };
    if (abs_tick & 32768u64) != 0 {
        ratio = (ratio * U256::from(66119101136024775622716233608466517926_u128)).shr(128);
    };
    if (abs_tick & 65536u64) != 0 {
        ratio = (ratio * U256::from(12847376061809297530290974190478138313_u128)).shr(128);
    };
    if (abs_tick & 131072u64) != 0 {
        ratio = (ratio * U256::from(485053260817066172746253684029974020_u128)).shr(128);
    };
    if (abs_tick & 262144u64) != 0 {
        ratio = (ratio * U256::from(691415978906521570653435304214168_u128)).shr(128);
    };
    if (abs_tick & 524288u64) != 0 {
        ratio = (ratio * U256::from(1404880482679654955896180642_u128)).shr(128);
    };

    if tick > 0 {
        ratio = U256::MAX / ratio;
    };

    if ratio.div_mod(U256::one().shl(32)).1 == U256::zero() {
        ratio.shr(32)
    } else {
        ratio.shr(32) + U256::one()
    }
}

pub fn get_tick_at_sqrt_ratio(sqrt_price_x96: U256) -> i32 {
    require(
        sqrt_price_x96 >= min_sqrt_ratio() && sqrt_price_x96 < max_sqrt_ratio(),
        common::error::Error::ErrR,
    );
    let ratio = sqrt_price_x96.shl(32);
    let mut r = ratio;
    let mut msb = 0u8;

    {
        let f = if r > U256::from(u128::MAX) {
            1_u8 << 7
        } else {
            0_u8 << 7
        };
        msb |= f;
        r = r.shr(f);
    }

    {
        let f = if r > U256::from(u64::MAX) {
            1_u8 << 6
        } else {
            0_u8 << 6
        };
        msb |= f;
        r = r.shr(f);
    }

    {
        let f = if r > U256::from(u32::MAX) {
            1_u8 << 5
        } else {
            0_u8 << 5
        };
        msb |= f;
        r = r.shr(f);
    }

    {
        let f = if r > U256::from(u16::MAX) {
            1_u8 << 4
        } else {
            0_u8 << 4
        };
        msb |= f;
        r = r.shr(f);
    }

    {
        let f = if r > U256::from(u8::MAX) {
            1_u8 << 3
        } else {
            0_u8 << 3
        };
        msb |= f;
        r = r.shr(f);
    }

    {
        let f = if r > U256::from(15) {
            1_u8 << 2
        } else {
            0_u8 << 2
        };
        msb |= f;
        r = r.shr(f);
    }

    {
        let f = if r > U256::from(3) {
            1_u8 << 1
        } else {
            0_u8 << 1
        };
        msb |= f;
        r = r.shr(f);
    }

    {
        let f = if r > U256::from(1) { 1 } else { 0 };
        msb |= f;
    }

    if msb >= 128 {
        r = ratio.shr(msb - 127);
    } else {
        r = ratio.shl(127 - msb);
    }

    let mut log_2 = (I256::from(msb) - I256::from(128)).shl(64.into());
    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(63)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(62)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(61)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(60)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(59)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(58)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(57)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(56)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(55)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(54)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(53)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(52)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(51)));
        r = r.shr(f);
    }

    {
        let t = u256_to_u512(r);
        r = u512_to_u256((t * t).shr(127));
        let f = r.shr(128);
        log_2 = log_2.bitor(I256::from(f.shl(50)));
    }
    let log_sqrt10001 = log_2 * I256::from(255738958999603826347141_u128);
    let t0 = I256::from("3402992956809132418596140100660247210");
    let t0 = (log_sqrt10001 - t0).shr(128.into());
    let tick_low: i32 = t0.0.as_i32();
    let t1 =
        (log_sqrt10001 + I256::from(291339464771989622907027621153398088495_u128)).shr(128.into());
    let tick_hi = t1.0.as_i32();
    if tick_low == tick_hi {
        tick_low
    } else if get_sqrt_ratio_at_tick(tick_hi) <= sqrt_price_x96 {
        tick_hi
    } else {
        tick_low
    }
}
