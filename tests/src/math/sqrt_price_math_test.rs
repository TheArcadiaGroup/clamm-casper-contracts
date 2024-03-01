#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate alloc;
use std::ops::{Div, Shl, Sub};

use super::test_math_session::*;
use crate::math::test_math_session::exec_test_math;
use crate::utils::*;
use casper_types::{runtime_args, RuntimeArgs, U128, U256, U512};
use math::tickmath::{self, max_sqrt_ratio, min_sqrt_ratio};
use types::i128::I128;

const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = 887272;

#[test]
#[should_panic = "User(15011)"]
fn test_get_next_sqrt_price_from_input_fail_price_zero() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => U256::one(),
            "1" => U128::from(0),
            "2" => expand_to_18_decimals(1).div(10),
            "3" => false
        },
        U256::zero(),
    );
}

#[test]
#[should_panic = "User(15011)"]
fn test_get_next_sqrt_price_from_input_fail_liquidity_zero() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => U256::one(),
            "1" => U128::from(0),
            "2" => expand_to_18_decimals(1).div(10),
            "3" => true
        },
        U256::zero(),
    );
}

#[test]
#[should_panic = "User(15012)"]
fn test_get_next_sqrt_price_from_input_fail_input_amount_overflow_price() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => U256::one().shl(160) - U256::from(1),
            "1" => U128::from(1024),
            "2" => U256::from(1024),
            "3" => false
        },
        U256::zero(),
    );
}

#[test]
fn test_get_next_sqrt_price_from_input_any_input_amount_cannot_underflow_price() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => U256::from(1),
            "1" => U128::from(1),
            "2" => U256::from(2).pow(255.into()),
            "3" => true
        },
        U256::one(),
    );
}

#[test]
fn test_get_next_sqrt_price_from_input_returns_input_price_amountin_zero_and_zeroforone_true() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => encode_price_sqrt(1, 1),
            "1" => U128::from(expand_to_18_decimals(1).div(U256::from(10)).as_u128()),
            "2" => U256::zero(),
            "3" => true
        },
        encode_price_sqrt(1, 1),
    );
}

#[test]
fn test_get_next_sqrt_price_from_input_returns_input_price_amountin_zero_and_zeroforone_false() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => encode_price_sqrt(1, 1),
            "1" => U128::from(expand_to_18_decimals(1).div(U256::from(10)).as_u128()),
            "2" => U256::zero(),
            "3" => false
        },
        encode_price_sqrt(1, 1),
    );
}

#[test]
fn test_get_next_sqrt_price_from_input_returns_minimum_price_for_max_inputs() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => U256::one().shl(160).sub(U256::one()),
            "1" => U128::MAX,
            "2" => U256::MAX.sub(U256::from(u128::MAX).shl(96).div(U256::one().shl(160).sub(U256::one()))),
            "3" => true
        },
        U256::one(),
    );
}

#[test]
fn test_get_next_sqrt_price_from_input_input_amount_0_dot_1_token1() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => encode_price_sqrt(1, 1),
            "1" => U128::from(expand_to_18_decimals(1).as_u128()),
            "2" => expand_to_18_decimals(1).div(U256::from(10)),
            "3" => false
        },
        U256::from_dec_str("87150978765690771352898345369").unwrap(),
    );
}

#[test]
fn test_get_next_sqrt_price_from_input_input_amount_0_dot_1_token0() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => encode_price_sqrt(1, 1),
            "1" => U128::from(expand_to_18_decimals(1).as_u128()),
            "2" => expand_to_18_decimals(1).div(U256::from(10)),
            "3" => true
        },
        U256::from_dec_str("72025602285694852357767227579").unwrap(),
    );
}

#[test]
fn test_get_next_sqrt_price_from_input_can_return_1_with_enough_amountin_zero_for_one_true() {
    exec_test_math(
        "get_next_sqrt_price_from_input",
        runtime_args! {
            "0" => encode_price_sqrt(1, 1),
            "1" => U128::one(),
            "2" => U256::MAX.div(2),
            "3" => true
        },
        U256::from_dec_str("1").unwrap(),
    );
}

#[cfg(test)]
mod get_next_sqrt_price_from_output {
    use super::*;
    #[test]
    #[should_panic = "User(15011)"]
    fn test_fail_if_price_zero() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => U256::zero(),
                "1" => U128::zero(),
                "2" => expand_to_18_decimals(1).div(10),
                "3" => false
            },
            U256::from_dec_str("1").unwrap(),
        );
    }

    #[test]
    #[should_panic = "User(15011)"]
    fn test_fail_if_liquidity_zero() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => U256::one(),
                "1" => U128::zero(),
                "2" => expand_to_18_decimals(1).div(10),
                "3" => true
            },
            U256::from_dec_str("1").unwrap(),
        );
    }

    #[test]
    #[should_panic = "User(15011)"]
    fn test_fails_output_amount_exactly_virtual_reserves_token0() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => U256::from_dec_str("20282409603651670423947251286016").unwrap(),
                "1" => U128::from(1024),
                "2" => U256::from(4),
                "3" => false
            },
            U256::from_dec_str("1").unwrap(),
        );
    }

    #[test]
    #[should_panic = "User(15011)"]
    fn test_fails_output_amount_greater_than_virtual_reserves_token0() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => U256::from_dec_str("20282409603651670423947251286016").unwrap(),
                "1" => U128::from(1024),
                "2" => U256::from(5),
                "3" => false
            },
            U256::from_dec_str("1").unwrap(),
        );
    }

    #[test]
    #[should_panic = "User(15011)"]
    fn test_fails_output_amount_greater_than_virtual_reserves_token1() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => U256::from_dec_str("20282409603651670423947251286016").unwrap(),
                "1" => U128::from(1024),
                "2" => U256::from(262145),
                "3" => true
            },
            U256::from_dec_str("1").unwrap(),
        );
    }

    #[test]
    #[should_panic = "User(15011)"]
    fn test_fails_output_amount_exactly_virtual_reserves_token1() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => U256::from_dec_str("20282409603651670423947251286016").unwrap(),
                "1" => U128::from(1024),
                "2" => U256::from(262144),
                "3" => true
            },
            U256::from_dec_str("1").unwrap(),
        );
    }

    #[test]
    fn test_succeeds_output_amount_just_less_than_virtual_reserves_token1() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => U256::from_dec_str("20282409603651670423947251286016").unwrap(),
                "1" => U128::from(1024),
                "2" => U256::from(262143),
                "3" => true
            },
            U256::from_dec_str("77371252455336267181195264").unwrap(),
        );
    }

    #[test]
    fn test_returns_input_price_amount_zero_and_zero_for_one_false() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => U128::from(expand_to_18_decimals(1).div(10).as_u128()),
                "2" => U256::from(0),
                "3" => false
            },
            encode_price_sqrt(1, 1),
        );
    }

    #[test]
    fn test_output_amount_0_dot_1_token1() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => U128::from(expand_to_18_decimals(1).as_u128()),
                "2" => expand_to_18_decimals(1).div(U256::from(10)),
                "3" => false
            },
            U256::from_dec_str("88031291682515930659493278152").unwrap(),
        );
    }

    #[test]
    fn test_output_amount_0_dot_1_token0() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => U128::from(expand_to_18_decimals(1).as_u128()),
                "2" => expand_to_18_decimals(1).div(U256::from(10)),
                "3" => true
            },
            U256::from_dec_str("71305346262837903834189555302").unwrap(),
        );
    }

    #[test]
    #[should_panic]
    fn test_reverts_amountout_impossible_zero_for_one_direction() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => U128::from(1),
                "2" => U256::MAX,
                "3" => true
            },
            U256::from_dec_str("71305346262837903834189555302").unwrap(),
        );
    }

    #[test]
    #[should_panic]
    fn test_reverts_amountout_impossible_one_for_zero_direction() {
        exec_test_math(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => U128::from(1),
                "2" => U256::MAX,
                "3" => false
            },
            U256::from_dec_str("71305346262837903834189555302").unwrap(),
        );
    }
}

#[cfg(test)]
mod get_amount0_delta {
    use super::*;
    #[test]
    fn test_return_0_if_liquidity_0() {
        exec_test_math(
            "get_amount0_delta",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => encode_price_sqrt(2, 1),
                "2" => U128::zero(),
                "3" => true
            },
            U256::from_dec_str("0").unwrap(),
        );
    }

    #[test]
    fn test_return_0_if_price_equal() {
        exec_test_math(
            "get_amount0_delta",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => encode_price_sqrt(1, 1),
                "2" => U128::zero(),
                "3" => true
            },
            U256::from_dec_str("0").unwrap(),
        );
    }

    #[test]
    fn test_return_correct() {
        exec_test_math(
            "get_amount0_delta",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => encode_price_sqrt(121, 100),
                "2" => U128::from(expand_to_18_decimals(1).as_u128()),
                "3" => true
            },
            U256::from_dec_str("90909090909090910").unwrap(),
        );

        exec_test_math(
            "get_amount0_delta",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => encode_price_sqrt(121, 100),
                "2" => U128::from(expand_to_18_decimals(1).as_u128()),
                "3" => false
            },
            U256::from_dec_str("90909090909090910")
                .unwrap()
                .sub(U256::one()),
        );
    }

    #[test]
    fn test_work_for_prices_overflow() {
        let amount0_up: U256 = get_math_lib_with_tc(
            "get_amount0_delta",
            runtime_args! {
                "0" => U256::one().shl(90).sub(1),
                "1" => U256::one().shl(96).sub(1),
                "2" => U128::from(expand_to_18_decimals(1).as_u128()),
                "3" => true
            },
        );

        let amount0_down: U256 = get_math_lib_with_tc(
            "get_amount0_delta",
            runtime_args! {
                "0" => U256::one().shl(90).sub(1),
                "1" => U256::one().shl(96).sub(1),
                "2" => U128::from(expand_to_18_decimals(1).as_u128()),
                "3" => false
            },
        );

        assert!(amount0_up == amount0_down + U256::one());
    }
}

#[cfg(test)]
mod get_amount1_delta {
    use super::*;
    #[test]
    fn test_return_0_if_liquidity_0() {
        exec_test_math(
            "get_amount1_delta",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => encode_price_sqrt(2, 1),
                "2" => U128::zero(),
                "3" => true
            },
            U256::from_dec_str("0").unwrap(),
        );
    }

    #[test]
    fn test_return_0_if_price_equal() {
        exec_test_math(
            "get_amount1_delta",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => encode_price_sqrt(1, 1),
                "2" => U128::zero(),
                "3" => true
            },
            U256::from_dec_str("0").unwrap(),
        );
    }

    #[test]
    fn test_return_correct() {
        exec_test_math(
            "get_amount1_delta",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => encode_price_sqrt(121, 100),
                "2" => U128::from(expand_to_18_decimals(1).as_u128()),
                "3" => true
            },
            U256::from_dec_str("100000000000000000").unwrap(),
        );

        exec_test_math(
            "get_amount1_delta",
            runtime_args! {
                "0" => encode_price_sqrt(1, 1),
                "1" => encode_price_sqrt(121, 100),
                "2" => U128::from(expand_to_18_decimals(1).as_u128()),
                "3" => false
            },
            U256::from_dec_str("100000000000000000")
                .unwrap()
                .sub(U256::one()),
        );
    }
}

#[cfg(test)]
mod swap_computation {
    use super::*;

    #[test]
    fn test_overflow() {
        let sqrt_q: U256 = get_math_lib_with_tc(
            "get_next_sqrt_price_from_input",
            runtime_args! {
                "0" => U256::from_dec_str("1025574284609383690408304870162715216695788925244").unwrap(),
                "1" => U128::from_dec_str("50015962439936049619261659728067971248").unwrap(),
                "2" => U256::from(406),
                "3" => true
            },
        );
        assert!(
            sqrt_q
                == U256::from_dec_str("1025574284609383582644711336373707553698163132913").unwrap()
        );

        exec_test_math(
            "get_amount0_delta",
            runtime_args! {
                "0" => U256::from_dec_str("1025574284609383582644711336373707553698163132913").unwrap(),
                "1" => U256::from_dec_str("1025574284609383690408304870162715216695788925244").unwrap(),
                "2" => U128::from_dec_str("50015962439936049619261659728067971248").unwrap(),
                "3" => true
            },
            U256::from_dec_str("406").unwrap(),
        );
    }
}
