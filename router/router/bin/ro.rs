use alloc::{boxed::Box, string::ToString, vec::Vec};
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{
    bytesrepr::{Bytes, FromBytes},
    CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, Key, Parameter, U128,
    U256,
};
use contract_utilities::helpers::get_named_args_3;
use router::periphery::{
    liquidity_amounts,
    logics::{
        burn_internal, collect_internal, create_and_initialize_pool_if_necessary_internal,
        decrease_liquidity_internal, increase_liquidity_internal, mint_callback_internal,
        mint_internal,
    },
    store::{
        get_next_id_ep, get_next_pool_id_ep, get_pool_id_ep, get_pool_key_ep, get_position_ep,
    },
    swap_router::{
        exact_input_internal, exact_input_single_internal, exact_output_internal,
        exact_output_single_internal, swap_callback_internal,
    },
};
use types::{
    i256::I256, CollectParams, CollectResult, DecreaseLiquidityParams, DecreaseLiquidityResult,
    ExactInputParams, ExactInputSingleParams, ExactOutputParams, ExactOutputSingleParams,
    IncreaseLiquidityParams, IncreaseLiquidityResult, MintParams, MintResult,
};

pub fn router_entry_points_list() -> Vec<EntryPoint> {
    let mut ret = vec![];
    ret.push(get_position_ep());
    ret.push(get_pool_id_ep());
    ret.push(get_pool_key_ep());
    ret.push(get_next_id_ep());
    ret.push(get_next_pool_id_ep());

    ret.push(EntryPoint::new(
        "mint",
        vec![Parameter::new("data", CLType::List(Box::new(CLType::U8)))],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "increase_liquidity",
        vec![Parameter::new("data", CLType::List(Box::new(CLType::U8)))],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "decrease_liquidity",
        vec![Parameter::new("data", CLType::List(Box::new(CLType::U8)))],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "collect",
        vec![Parameter::new("data", CLType::List(Box::new(CLType::U8)))],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "burn",
        vec![Parameter::new("token_id", CLType::U256)],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "swap_callback",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "exact_input_single",
        vec![Parameter::new("data", CLType::List(Box::new(CLType::U8)))],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "exact_input",
        vec![Parameter::new("data", CLType::List(Box::new(CLType::U8)))],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "get_liquidity_for_amount0",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "get_liquidity_for_amount1",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "get_liquidity_for_amounts",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "get_amount0_for_liquidity",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "get_amount1_for_liquidity",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "get_amounts_for_liquidity",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "create_and_initialize_pool_if_necessary",
        vec![
            Parameter::new("token0", Key::cl_type()),
            Parameter::new("token1", Key::cl_type()),
            Parameter::new("fee", CLType::U32),
            Parameter::new("sqrt_price_x96", CLType::U256),
        ],
        Key::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "mint_callback",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "decode_first_pool",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "get_first_pool",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "skip_token",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret.push(EntryPoint::new(
        "has_multiple_pools",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    ret
}

#[no_mangle]
pub fn mint() {
    let data: Bytes = runtime::get_named_arg("data");
    let mint_params = MintParams::from_bytes(&data).unwrap().0;
    let (token_id, liquidity, amount0, amount1) = mint_internal(&mint_params);
    runtime::ret(
        CLValue::from_t(MintResult {
            token_id,
            amount0,
            amount1,
            liquidity: liquidity.into(),
        })
        .unwrap_or_revert(),
    );
}

#[no_mangle]
pub fn increase_liquidity() {
    let data: Bytes = runtime::get_named_arg("data");
    let params = IncreaseLiquidityParams::from_bytes(&data).unwrap().0;
    let (liquidity, amount0, amount1) = increase_liquidity_internal(&params);
    runtime::ret(
        CLValue::from_t(IncreaseLiquidityResult {
            amount0,
            amount1,
            liquidity: liquidity.into(),
        })
        .unwrap_or_revert(),
    );
}

#[no_mangle]
pub fn decrease_liquidity() {
    let data: Bytes = runtime::get_named_arg("data");
    let params = DecreaseLiquidityParams::from_bytes(&data).unwrap().0;
    let (amount0, amount1) = decrease_liquidity_internal(&params);
    runtime::ret(CLValue::from_t(DecreaseLiquidityResult { amount0, amount1 }).unwrap_or_revert());
}

#[no_mangle]
pub fn collect() {
    let data: Bytes = runtime::get_named_arg("data");
    let params = CollectParams::from_bytes(&data).unwrap().0;
    let (token0, token1, amount0, amount1) = collect_internal(&params);
    runtime::ret(
        CLValue::from_t(CollectResult {
            amount0,
            amount1,
            token0,
            token1,
        })
        .unwrap_or_revert(),
    );
}

#[no_mangle]
pub fn burn() {
    let token_id: U256 = runtime::get_named_arg("token_id");
    burn_internal(token_id);
}

#[no_mangle]
pub fn swap_callback() {
    let (amount0_delta, amount1_delta, data): (I256, I256, Bytes) = get_named_args_3(vec![
        "amount0_delta".to_string(),
        "amount1_delta".to_string(),
        "data".to_string(),
    ]);
    swap_callback_internal(amount0_delta.into(), amount1_delta.into(), &data);
}

#[no_mangle]
pub fn exact_input_single() {
    let data: Bytes = runtime::get_named_arg("data");
    let params = ExactInputSingleParams::from_bytes(&data).unwrap().0;
    let amount_out = exact_input_single_internal(&params);
    runtime::ret(CLValue::from_t(amount_out).unwrap_or_revert());
}

#[no_mangle]
pub fn exact_input() {
    let data: Bytes = runtime::get_named_arg("data");
    let params = ExactInputParams::from_bytes(&data).unwrap().0;
    let amount_out = exact_input_internal(&params);
    runtime::ret(CLValue::from_t(amount_out).unwrap_or_revert());
}

#[no_mangle]
pub fn exact_output_single() {
    let data: Bytes = runtime::get_named_arg("data");
    let params = ExactOutputSingleParams::from_bytes(&data).unwrap().0;
    let amount_in = exact_output_single_internal(&params);
    runtime::ret(CLValue::from_t(amount_in).unwrap_or_revert());
}

#[no_mangle]
pub fn exact_output() {
    let data: Bytes = runtime::get_named_arg("data");
    let params = ExactOutputParams::from_bytes(&data).unwrap().0;
    let amount_in = exact_output_internal(&params);
    runtime::ret(CLValue::from_t(amount_in).unwrap_or_revert());
}

#[no_mangle]
pub fn get_liquidity_for_amount0() {
    let sqrt_ratio_a_x96: U256 = runtime::get_named_arg("sqrt_ratio_a_x96");
    let sqrt_ratio_b_x96: U256 = runtime::get_named_arg("sqrt_ratio_b_x96");
    let amount0: U256 = runtime::get_named_arg("amount0");
    let ret = liquidity_amounts::get_liquidity_for_amount0(
        &sqrt_ratio_a_x96,
        &sqrt_ratio_b_x96,
        &amount0,
    );
    runtime::ret(CLValue::from_t(U128::from(ret)).unwrap_or_revert());
}

#[no_mangle]
pub fn get_liquidity_for_amount1() {
    let sqrt_ratio_a_x96: U256 = runtime::get_named_arg("sqrt_ratio_a_x96");
    let sqrt_ratio_b_x96: U256 = runtime::get_named_arg("sqrt_ratio_b_x96");
    let amount1: U256 = runtime::get_named_arg("amount1");
    let ret = liquidity_amounts::get_liquidity_for_amount1(
        &sqrt_ratio_a_x96,
        &sqrt_ratio_b_x96,
        &amount1,
    );
    runtime::ret(CLValue::from_t(U128::from(ret)).unwrap_or_revert());
}

#[no_mangle]
pub fn get_liquidity_for_amounts() {
    let sqrt_ratio_a_x96: U256 = runtime::get_named_arg("sqrt_ratio_a_x96");
    let sqrt_ratio_b_x96: U256 = runtime::get_named_arg("sqrt_ratio_b_x96");
    let sqrt_ratio_x96: U256 = runtime::get_named_arg("sqrt_ratio_x96");
    let amount1: U256 = runtime::get_named_arg("amount1");
    let amount0: U256 = runtime::get_named_arg("amount0");
    let ret = liquidity_amounts::get_liquidity_for_amounts(
        &sqrt_ratio_x96,
        &sqrt_ratio_a_x96,
        &sqrt_ratio_b_x96,
        &amount0,
        &amount1,
    );
    runtime::ret(CLValue::from_t(U128::from(ret)).unwrap_or_revert());
}

#[no_mangle]
pub fn get_amount0_for_liquidity() {
    let sqrt_ratio_a_x96: U256 = runtime::get_named_arg("sqrt_ratio_a_x96");
    let sqrt_ratio_b_x96: U256 = runtime::get_named_arg("sqrt_ratio_b_x96");
    let liquidity: U128 = runtime::get_named_arg("liquidity");
    let ret = liquidity_amounts::get_amount0_for_liquidity(
        &sqrt_ratio_a_x96,
        &sqrt_ratio_b_x96,
        liquidity.as_u128(),
    );
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
pub fn get_amount1_for_liquidity() {
    let sqrt_ratio_a_x96: U256 = runtime::get_named_arg("sqrt_ratio_a_x96");
    let sqrt_ratio_b_x96: U256 = runtime::get_named_arg("sqrt_ratio_b_x96");
    let liquidity: U128 = runtime::get_named_arg("liquidity");
    let ret = liquidity_amounts::get_amount1_for_liquidity(
        &sqrt_ratio_a_x96,
        &sqrt_ratio_b_x96,
        liquidity.as_u128(),
    );
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
pub fn get_amounts_for_liquidity() {
    let sqrt_ratio_a_x96: U256 = runtime::get_named_arg("sqrt_ratio_a_x96");
    let sqrt_ratio_b_x96: U256 = runtime::get_named_arg("sqrt_ratio_b_x96");
    let sqrt_ratio_x96: U256 = runtime::get_named_arg("sqrt_ratio_x96");
    let liquidity: U128 = runtime::get_named_arg("liquidity");
    let ret = liquidity_amounts::get_amounts_for_liquidity(
        &sqrt_ratio_x96,
        &sqrt_ratio_a_x96,
        &sqrt_ratio_b_x96,
        liquidity.as_u128(),
    );
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
pub fn create_and_initialize_pool_if_necessary() {
    let token0: Key = runtime::get_named_arg("token0");
    let token1: Key = runtime::get_named_arg("token1");
    let fee: u32 = runtime::get_named_arg("fee");
    let sqrt_price_x96: U256 = runtime::get_named_arg("sqrt_price_x96");
    runtime::ret(
        CLValue::from_t(create_and_initialize_pool_if_necessary_internal(
            token0,
            token1,
            fee,
            &sqrt_price_x96,
        ))
        .unwrap_or_revert(),
    )
}

#[no_mangle]
pub extern "C" fn mint_callback() {
    let (amount0_owed, amount1_owed, data): (U256, U256, Bytes) = get_named_args_3(vec![
        "amount0_owed".to_string(),
        "amount1_owed".to_string(),
        "data".to_string(),
    ]);
    mint_callback_internal(amount0_owed, amount1_owed, &data);
}

#[no_mangle]
pub fn decode_first_pool() {
    let path: Bytes = runtime::get_named_arg("path");
    let ret = common::path::decode_first_pool(&path);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
pub fn get_first_pool() {
    let path: Bytes = runtime::get_named_arg("path");
    let ret = common::path::get_first_pool(&path);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
pub fn skip_token() {
    let path: Bytes = runtime::get_named_arg("path");
    let ret = common::path::skip_token(&path);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
pub fn has_multiple_pools() {
    let path: Bytes = runtime::get_named_arg("path");
    let ret = common::path::has_multiple_pools(&path);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}