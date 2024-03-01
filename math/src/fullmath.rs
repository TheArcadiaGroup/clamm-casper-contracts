use casper_types::U256;
use common::error::require;
use contract_utilities::helpers::{self, u256_to_u512, u512_to_u256};
use core::ops::Shl;
use core::ops::{BitAnd, BitOr, BitXor, Div};

pub fn mul_div(a: &U256, b: &U256, denominator: &U256) -> U256 {
    // 512-bit multiply [prod1 prod0] = a * b
    // Compute the product mod 2**256 and mod 2**256 - 1
    // then use the Chinese Remainder Theorem to reconstruct
    // the 512 bit result. The result is stored in two 256
    // variables such that product = prod1 * 2**256 + prod0
    let product = helpers::u256_to_u512(*a) * helpers::u256_to_u512(*b);
    let mm = product.div_mod(helpers::u256_to_u512(U256::MAX)).1;
    let mm = u512_to_u256(mm);
    let mut prod0 = a.overflowing_mul(*b).0;
    let mut prod1 = mm
        .overflowing_sub(prod0)
        .0
        .overflowing_sub(if mm < prod0 {
            U256::one()
        } else {
            U256::zero()
        })
        .0;
    if prod1 == U256::zero() {
        require(
            denominator > &U256::zero(),
            common::error::Error::InvalidMulDiv,
        );
        return prod0.div(denominator);
    }

    require(denominator > &prod1, common::error::Error::InvalidMulDiv);
    let remainder = u512_to_u256(product.div_mod(u256_to_u512(*denominator)).1);
    prod1 = prod1
        .overflowing_sub(if remainder > prod0 {
            U256::from(1)
        } else {
            0.into()
        })
        .0;
    prod0 = prod0.overflowing_sub(remainder).0;

    let mut twos = U256::zero()
        .overflowing_sub(*denominator)
        .0
        .bitand(*denominator);
    let denominator: U256 = denominator.div(twos);
    prod0 = prod0.div(twos);

    twos = U256::zero()
        .overflowing_sub(twos)
        .0
        .div(twos)
        .overflowing_add(U256::one())
        .0;
    prod0 = prod0.bitor(prod1.overflowing_mul(twos).0);

    let mut inv = denominator.overflowing_mul(3.into()).0.bitxor(2.into());
    inv = inv
        .overflowing_mul(
            U256::from(2)
                .overflowing_sub(denominator.overflowing_mul(inv).0)
                .0,
        )
        .0;
    inv = inv
        .overflowing_mul(
            U256::from(2)
                .overflowing_sub(denominator.overflowing_mul(inv).0)
                .0,
        )
        .0;
    inv = inv
        .overflowing_mul(
            U256::from(2)
                .overflowing_sub(denominator.overflowing_mul(inv).0)
                .0,
        )
        .0;
    inv = inv
        .overflowing_mul(
            U256::from(2)
                .overflowing_sub(denominator.overflowing_mul(inv).0)
                .0,
        )
        .0;
    inv = inv
        .overflowing_mul(
            U256::from(2)
                .overflowing_sub(denominator.overflowing_mul(inv).0)
                .0,
        )
        .0;
    inv = inv
        .overflowing_mul(
            U256::from(2)
                .overflowing_sub(denominator.overflowing_mul(inv).0)
                .0,
        )
        .0;

    prod0.overflowing_mul(inv).0
}

pub fn mul_mod(a: &U256, b: &U256, denominator: &U256) -> U256 {
    helpers::u512_to_u256(
        (helpers::u256_to_u512(*a) * helpers::u256_to_u512(*b))
            .div_mod(helpers::u256_to_u512(*denominator))
            .1,
    )
}

pub fn mul_div_rounding_up(a: &U256, b: &U256, denominator: &U256) -> U256 {
    let mut result = mul_div(a, b, denominator);
    if mul_mod(a, b, denominator) > U256::zero() {
        require(result < U256::MAX, common::error::Error::InvalidMulDiv);
        result = result.overflowing_add(U256::one()).0;
    }
    result
}

pub fn unsafe_div_rounding_up(x: &U256, y: &U256) -> U256 {
    let div = x / y;
    let mo = x.div_mod(*y).1;
    let quotient = if mo != U256::zero() {
        U256::one()
    } else {
        U256::zero()
    };
    div.overflowing_add(quotient).0
}

pub fn overflow_sub_u160(a: &U256, b: &U256) -> U256 {
    if a.lt(b) {
        a + U256::one().shl(160) - b
    } else {
        a - b
    }
}
