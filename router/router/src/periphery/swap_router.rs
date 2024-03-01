use core::ops::{Add, Sub};

use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    Key, U256,
};
use common::{
    erc20_helpers,
    error::require,
    intf::swap,
    path::{decode_first_pool, get_first_pool, has_multiple_pools, skip_token},
    utils::is_token_sorted,
};
use contract_utilities::helpers::{self, get_immediate_caller_key, get_self_key, null_key};
use math::tickmath;
use types::{
    i256::I256, ExactInputParams, ExactInputSingleParams, ExactOutputParams,
    ExactOutputSingleParams, SwapCallbackData,
};

use super::{
    checks::check_deadline,
    logics::_verify_callback,
    pool_key::{self, get_pool_address},
    store::{read_amount_in_cached, save_amount_in_cached, DEFAULT_AMOUNT_IN_CACHED},
};

pub fn swap_callback_internal(amount0_delta: I256, amount1_delta: I256, _data: &[u8]) {
    require(
        amount0_delta.0 > 0 || amount1_delta.0 > 0,
        common::error::Error::ErrInvalidSwapCallbackParams,
    );
    let mut data = SwapCallbackData::from_bytes(_data)
        .unwrap_or_revert_with(common::error::Error::ErrInvalidSwapCallbackParams)
        .0;

    let (mut token_in, token_out, fee) = decode_first_pool(&data.path);

    _verify_callback(token_in, token_out, fee);

    let (is_exact_input, amount_to_pay) = if amount0_delta.0 > 0 {
        (
            is_token_sorted(token_in, token_out),
            U256::from(amount0_delta),
        )
    } else {
        (
            is_token_sorted(token_out, token_in),
            U256::from(amount1_delta),
        )
    };

    if is_exact_input {
        erc20_helpers::transfer_from(
            token_in,
            data.payer,
            get_immediate_caller_key(),
            amount_to_pay,
        );
    } else {
        if has_multiple_pools(&data.path) {
            data.path = skip_token(&data.path);

            // call exact output internal
        } else {
            save_amount_in_cached(amount_to_pay);
            token_in = token_out;
            erc20_helpers::transfer_from(
                token_in,
                data.payer,
                get_immediate_caller_key(),
                amount_to_pay,
            );
        }
    }
}

pub fn _exact_input_internal(
    amount_in: U256,
    recipient: Key,
    sqrt_price_limit_x96: U256,
    data: &SwapCallbackData,
) -> U256 {
    let recipient = if recipient == null_key() {
        get_self_key()
    } else {
        recipient
    };

    let (token_in, token_out, fee) = decode_first_pool(&data.path);
    let zero_for_one = is_token_sorted(token_in, token_out);

    let pool_key = pool_key::get_pool_key(token_in, token_out, fee);
    let pool = get_pool_address(&pool_key);

    let (amount0, amount1) = swap(
        pool,
        recipient,
        zero_for_one,
        amount_in,
        true,
        if sqrt_price_limit_x96.is_zero() {
            if zero_for_one {
                tickmath::min_sqrt_ratio().add(1)
            } else {
                tickmath::max_sqrt_ratio().sub(1)
            }
        } else {
            sqrt_price_limit_x96
        },
        data.to_bytes().unwrap(),
    );

    if zero_for_one {
        U256::from(-amount1)
    } else {
        U256::from(-amount0)
    }
}

pub fn exact_input_single_internal(params: &ExactInputSingleParams) -> U256 {
    check_deadline(params.deadline);

    let amount_out = _exact_input_internal(
        params.amount_in,
        params.recipient,
        params.sqrt_price_limit_x96,
        &SwapCallbackData {
            path: helpers::encode_3(&params.token_out, &params.fee, &params.token_in).into(),
            payer: get_immediate_caller_key(),
        },
    );
    require(
        amount_out.ge(&params.amount_out_minimum),
        common::error::Error::ErrTooLittleReceived,
    );
    amount_out
}

pub fn exact_input_internal(params: &ExactInputParams) -> U256 {
    check_deadline(params.deadline);
    let mut payer = get_immediate_caller_key();
    let mut params = params.clone();
    let amount_out;
    loop {
        let has_multiple_pools_ = has_multiple_pools(&params.path);
        params.amount_in = _exact_input_internal(
            params.amount_in,
            if has_multiple_pools_ {
                get_self_key()
            } else {
                params.recipient
            },
            0.into(),
            &SwapCallbackData {
                path: get_first_pool(&params.path),
                payer,
            },
        );

        if has_multiple_pools_ {
            payer = get_self_key();
            params.path = skip_token(&params.path);
        } else {
            amount_out = params.amount_in;
            break;
        }
    }
    require(
        amount_out.ge(&params.amount_out_minimum),
        common::error::Error::ErrTooLittleReceived,
    );
    amount_out
}

pub fn _exact_output_internal(
    amount_out: U256,
    recipient: Key,
    sqrt_price_limit_x96: U256,
    data: &SwapCallbackData,
) -> U256 {
    let recipient = if recipient == null_key() {
        get_self_key()
    } else {
        recipient
    };

    let (token_in, token_out, fee) = decode_first_pool(&data.path);
    let zero_for_one = is_token_sorted(token_in, token_out);

    let pool_key = pool_key::get_pool_key(token_in, token_out, fee);
    let pool = get_pool_address(&pool_key);

    let (amount0_delta, amount1_delta) = swap(
        pool,
        recipient,
        zero_for_one,
        amount_out,
        false,
        if sqrt_price_limit_x96.is_zero() {
            if zero_for_one {
                tickmath::min_sqrt_ratio().add(1)
            } else {
                tickmath::max_sqrt_ratio().sub(1)
            }
        } else {
            sqrt_price_limit_x96
        },
        data.to_bytes().unwrap(),
    );
    let (amount_in, amount_out_received) = if zero_for_one {
        (U256::from(amount0_delta), U256::from(-amount1_delta))
    } else {
        (U256::from(amount1_delta), U256::from(-amount0_delta))
    };

    if sqrt_price_limit_x96.is_zero() {
        require(
            amount_out_received.eq(&amount_out),
            common::error::Error::ErrInvalidAmountOut,
        );
    }
    amount_in
}

pub fn exact_output_single_internal(params: &ExactOutputSingleParams) -> U256 {
    check_deadline(params.deadline);

    let amount_in = _exact_output_internal(
        params.amount_out,
        params.recipient,
        params.sqrt_price_limit_x96,
        &SwapCallbackData {
            path: helpers::encode_3(&params.token_out, &params.fee, &params.token_in).into(),
            payer: get_immediate_caller_key(),
        },
    );
    require(
        amount_in.le(&params.amount_in_maximum),
        common::error::Error::ErrTooMuchRequested,
    );
    save_amount_in_cached(DEFAULT_AMOUNT_IN_CACHED);
    amount_in
}

pub fn exact_output_internal(params: &ExactOutputParams) -> U256 {
    check_deadline(params.deadline);
    _exact_output_internal(
        params.amount_out,
        params.recipient,
        U256::zero(),
        &SwapCallbackData {
            path: params.path.clone(),
            payer: get_immediate_caller_key(),
        },
    );

    let amount_in = read_amount_in_cached();
    require(
        amount_in.le(&params.amount_in_maximum),
        common::error::Error::ErrTooMuchRequested,
    );
    save_amount_in_cached(DEFAULT_AMOUNT_IN_CACHED);
    amount_in
}
