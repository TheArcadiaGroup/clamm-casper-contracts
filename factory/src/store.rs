use alloc::{string::String, vec};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, Key, Parameter, U128, U256,
};
use common::{error::Error, get_set_dict, get_set_no_set};
use contract_utilities::helpers::{self, null_key};
use types::{Observation, PositionInfo, ProtocolFees, Slot0, TickInfo};

get_set_no_set!(
    factory,
    "factory",
    Key,
    null_key(),
    save_factory,
    read_factory,
    get_factory,
    get_factory_ep,
    "get_factory"
);

get_set_no_set!(
    token0,
    "token0",
    Key,
    null_key(),
    save_token0,
    read_token0,
    get_token0,
    get_token0_ep,
    "get_token0"
);

get_set_no_set!(
    token1,
    "token1",
    Key,
    null_key(),
    save_token1,
    read_token1,
    get_token1,
    get_token1_ep,
    "get_token1"
);

get_set_no_set!(fee, "fee", u32, 0, save_fee, read_fee, get_fee, get_fee_ep, "get_fee");

get_set_no_set!(
    tick_spacing,
    "tick_spacing",
    i32,
    0,
    save_tick_spacing,
    read_tick_spacing,
    get_tick_spacing,
    get_tick_spacing_ep,
    "get_tick_spacing"
);

get_set_no_set!(
    max_liquidity_per_tick,
    "max_liquidity_per_tick",
    U128,
    U128::zero(),
    save_max_liquidity_per_tick,
    read_max_liquidity_per_tick,
    get_max_liquidity_per_tick,
    get_max_liquidity_per_tick_ep,
    "get_max_liquidity_per_tick"
);

get_set_no_set!(
    slot0,
    "slot0",
    Slot0,
    Slot0::default(),
    save_slot0,
    read_slot0,
    get_slot0,
    get_slot0_ep,
    "get_slot0"
);

get_set_no_set!(
    slot0_unlocked,
    "slot0_unlocked",
    bool,
    false,
    save_slot0_unlocked,
    read_slot0_unlocked,
    get_slot0_unlocked,
    get_slot0_unlocked_ep,
    "get_slot0_unlocked"
);

get_set_no_set!(
    fee_growth_global0_x128,
    "fee_growth_global0_x128",
    U256,
    U256::zero(),
    save_fee_growth_global0_x128,
    read_fee_growth_global0_x128,
    get_fee_growth_global0_x128,
    get_fee_growth_global0_x128_ep,
    "get_fee_growth_global0_x128"
);

get_set_no_set!(
    fee_growth_global1_x128,
    "fee_growth_global1_x128",
    U256,
    U256::zero(),
    save_fee_growth_global1_x128,
    read_fee_growth_global1_x128,
    get_fee_growth_global1_x128,
    get_fee_growth_global1_x128_ep,
    "get_fee_growth_global1_x128"
);

get_set_no_set!(
    protocol_fees,
    "protocol_fees",
    ProtocolFees,
    ProtocolFees::default(),
    save_protocol_fees,
    read_protocol_fees,
    get_protocol_fees,
    get_protocol_fees_ep,
    "get_protocol_fees"
);

get_set_no_set!(
    liquidity,
    "liquidity",
    U128,
    U128::default(),
    save_liquidity,
    read_liquidity,
    get_liquidity,
    get_liquidity_ep,
    "get_liquidity"
);

get_set_dict!(
    "ticks",
    "tick",
    i32,
    TickInfo,
    TickInfo::default(),
    save_tick,
    read_tick,
    get_tick,
    get_tick_ep,
    "get_tick"
);

get_set_dict!(
    "tick_bitmap",
    "tick",
    i32,
    U256,
    U256::default(),
    save_tick_bitmap,
    read_tick_bitmap,
    get_tick_bitmap,
    get_tick_bitmap_ep,
    "get_tick_bitmap"
);

get_set_dict!(
    "positions",
    "position_key",
    String,
    PositionInfo,
    PositionInfo::default(),
    save_position,
    read_position,
    get_position,
    get_position_ep,
    "get_position"
);

get_set_dict!(
    "observations",
    "index",
    u64,
    Observation,
    Observation::default(),
    save_observation,
    read_observation,
    get_observation,
    get_observation_ep,
    "get_observation"
);

pub fn initialize() {
    storage::new_dictionary("observations").unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary("positions").unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary("tick_bitmap").unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary("ticks").unwrap_or_revert_with(Error::FailedToCreateDictionary);
    save_factory(helpers::null_key());
    save_token0(helpers::null_key());
    save_token1(helpers::null_key());
    save_fee(u32::default());
    save_tick_spacing(i32::default());
    save_max_liquidity_per_tick(U128::default());
    save_slot0(Slot0::default());
    // let slot0 = read_slot0();
    save_fee_growth_global0_x128(U256::default());
    save_fee_growth_global1_x128(U256::default());
    save_protocol_fees(ProtocolFees::default());
    save_liquidity(U128::default());
    save_tick(&0, &TickInfo::default());
}
