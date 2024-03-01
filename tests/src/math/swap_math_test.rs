#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use casper_types::{runtime_args, RuntimeArgs, U128, U256};
use std::ops::Add;

#[cfg(test)]
mod compute_swap_step {
    use std::ops::{Div, Mul, Sub};

    use types::{i128::I128, i256::I256};

    use super::*;
    use crate::{
        math::test_math_session::{get_math_lib_with_tc, get_swap_math_result},
        utils::{encode_price_sqrt, expand_to_18_decimals},
    };

    #[test]
    fn exact_amount_in_that_gets_capped_price_target_one_for_zero() {
        let price = encode_price_sqrt(1, 1);
        let price_target = encode_price_sqrt(101, 100);
        let liquidity = expand_to_18_decimals(2).as_u128();
        let amount = expand_to_18_decimals(1);
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            get_swap_math_result(price, price_target, liquidity, amount.into(), fee);
        assert!(amount_in == U256::from_dec_str("9975124224178055").unwrap());
        assert!(fee_amount == U256::from_dec_str("5988667735148").unwrap());
        assert!(amount_out == U256::from_dec_str("9925619580021728").unwrap());
        assert!(amount_in.add(fee_amount) < amount);

        let price_after_whole_input_amount: U256 = get_math_lib_with_tc(
            "get_next_sqrt_price_from_input",
            runtime_args! {
                "0" => price,
                "1" => U128::from(liquidity),
                "2" => amount,
                "3" => zero_for_one
            },
        );
        assert!(sqrt_q == price_target);
        assert!(sqrt_q < price_after_whole_input_amount);
    }

    #[test]
    fn exact_amount_out_that_gets_capped_price_target_one_for_zero() {
        let price = encode_price_sqrt(1, 1);
        let price_target = encode_price_sqrt(101, 100);
        let liquidity = expand_to_18_decimals(2).as_u128();
        let amount = I256::from(expand_to_18_decimals(1)).mul(I256::from(-1));
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            get_swap_math_result(price, price_target, liquidity, amount, fee);
        assert!(amount_in == U256::from_dec_str("9975124224178055").unwrap());
        assert!(fee_amount == U256::from_dec_str("5988667735148").unwrap());
        assert!(amount_out == U256::from_dec_str("9925619580021728").unwrap());
        assert!(amount_out < amount.mul(I256::from(-1)).into());

        let price_after_whole_input_amount: U256 = get_math_lib_with_tc(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => price,
                "1" => U128::from(liquidity),
                "2" => U256::from(amount.mul(I256::from(-1))),
                "3" => zero_for_one
            },
        );
        assert!(sqrt_q == price_target);
        assert!(sqrt_q < price_after_whole_input_amount);
    }

    #[test]
    fn exact_amount_in_that_fully_spent_one_for_zero() {
        let price = encode_price_sqrt(1, 1);
        let price_target = encode_price_sqrt(1000, 100);
        let liquidity = expand_to_18_decimals(2).as_u128();
        let amount = I256::from(expand_to_18_decimals(1));
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            get_swap_math_result(price, price_target, liquidity, amount, fee);
        assert!(amount_in == U256::from_dec_str("999400000000000000").unwrap());
        assert!(fee_amount == U256::from_dec_str("600000000000000").unwrap());
        assert!(amount_out == U256::from_dec_str("666399946655997866").unwrap());
        assert!(amount_in.add(fee_amount) == amount.into());

        let price_after_whole_input_amount: U256 = get_math_lib_with_tc(
            "get_next_sqrt_price_from_input",
            runtime_args! {
                "0" => price,
                "1" => U128::from(liquidity),
                "2" => U256::from(amount).sub(fee_amount),
                "3" => zero_for_one
            },
        );
        assert!(sqrt_q < price_target);
        assert!(sqrt_q == price_after_whole_input_amount);
    }

    #[test]
    fn exact_amount_out_that_fully_received_one_for_zero() {
        let price = encode_price_sqrt(1, 1);
        let price_target = encode_price_sqrt(1000, 100);
        let liquidity = expand_to_18_decimals(2).as_u128();
        let amount = I256::from(expand_to_18_decimals(1)).mul(I256::from(-1));
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            get_swap_math_result(price, price_target, liquidity, amount, fee);
        assert!(amount_in == U256::from_dec_str("2000000000000000000").unwrap());
        assert!(fee_amount == U256::from_dec_str("1200720432259356").unwrap());
        assert!(amount_out == amount.mul(I256::from(-1)).into());

        let price_after_whole_input_amount: U256 = get_math_lib_with_tc(
            "get_next_sqrt_price_from_output",
            runtime_args! {
                "0" => price,
                "1" => U128::from(liquidity),
                "2" => U256::from(amount.mul(I256::from(-1))),
                "3" => zero_for_one
            },
        );
        assert!(sqrt_q < price_target);
        assert!(sqrt_q == price_after_whole_input_amount);
    }

    #[test]
    fn exact_amount_out_capped_desired_amount_out() {
        let (sqrt_q, amount_in, amount_out, fee_amount) = get_swap_math_result(
            U256::from_dec_str("417332158212080721273783715441582").unwrap(),
            U256::from_dec_str("1452870262520218020823638996").unwrap(),
            159344665391607089467575320103,
            I256::from(-1),
            1,
        );
        assert!(amount_in == U256::from_dec_str("1").unwrap());
        assert!(fee_amount == U256::from_dec_str("1").unwrap());
        assert!(amount_out == U256::from(1));
        assert!(sqrt_q == U256::from_dec_str("417332158212080721273783715441581").unwrap());
    }

    #[test]
    fn target_price_1_uses_partial_input_amount() {
        let (sqrt_q, amount_in, amount_out, fee_amount) = get_swap_math_result(
            U256::from_dec_str("2").unwrap(),
            U256::from_dec_str("1").unwrap(),
            1,
            I256::from(3915081100057732413702495386755767_u128),
            1,
        );
        assert!(amount_in == U256::from_dec_str("39614081257132168796771975168").unwrap());
        assert!(fee_amount == U256::from_dec_str("39614120871253040049813").unwrap());
        assert!(
            amount_in.add(fee_amount)
                <= U256::from_dec_str("3915081100057732413702495386755767").unwrap()
        );
        assert!(amount_out == U256::from(0));
        assert!(sqrt_q == U256::from_dec_str("1").unwrap());
    }

    #[test]
    fn entire_input_amount_taken_fee() {
        let (sqrt_q, amount_in, amount_out, fee_amount) = get_swap_math_result(
            U256::from_dec_str("2413").unwrap(),
            U256::from_dec_str("79887613182836312").unwrap(),
            1985041575832132834610021537970,
            I256::from(10),
            1872,
        );
        assert!(amount_in == U256::from_dec_str("0").unwrap());
        assert!(fee_amount == U256::from_dec_str("10").unwrap());
        assert!(amount_out == U256::from(0));
        assert!(sqrt_q == U256::from_dec_str("2413").unwrap());
    }

    #[test]
    fn handle_intermediate_insufficient_liquidity_zero_for_one_exact_output_case() {
        let (sqrt_q, amount_in, amount_out, fee_amount) = get_swap_math_result(
            U256::from_dec_str("20282409603651670423947251286016").unwrap(),
            U256::from_dec_str("20282409603651670423947251286016")
                .unwrap()
                .mul(U256::from(11))
                .div(U256::from(10)),
            1024,
            I256::from(-4),
            3000,
        );
        assert!(amount_in == U256::from_dec_str("26215").unwrap());
        assert!(fee_amount == U256::from_dec_str("79").unwrap());
        assert!(amount_out == U256::from(0));
        assert!(
            sqrt_q
                == U256::from_dec_str("20282409603651670423947251286016")
                    .unwrap()
                    .mul(U256::from(11))
                    .div(U256::from(10))
        );
    }

    #[test]
    fn handle_intermediate_insufficient_liquidity_one_for_zero_exact_output_case() {
        let (sqrt_q, amount_in, amount_out, fee_amount) = get_swap_math_result(
            U256::from_dec_str("20282409603651670423947251286016").unwrap(),
            U256::from_dec_str("20282409603651670423947251286016")
                .unwrap()
                .mul(U256::from(9))
                .div(U256::from(10)),
            1024,
            I256::from(-263000),
            3000,
        );
        assert!(amount_in == U256::from_dec_str("1").unwrap());
        assert!(fee_amount == U256::from_dec_str("1").unwrap());
        assert!(amount_out == U256::from(26214));
        assert!(
            sqrt_q
                == U256::from_dec_str("20282409603651670423947251286016")
                    .unwrap()
                    .mul(U256::from(9))
                    .div(U256::from(10))
        );
    }
}
