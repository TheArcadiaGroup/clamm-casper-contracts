use alloc::{boxed::Box, string::String, vec, vec::Vec};
use casper_types::{
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

use common::{owner, timestamp_testing};
use types::ProtocolFees;

use crate::{
    factory::fac::{get_fee_amount_tick_spacing_ep, get_pool_map_ep},
    store::{
        get_factory_ep, get_fee_ep, get_fee_growth_global0_x128_ep, get_fee_growth_global1_x128_ep,
        get_liquidity_ep, get_max_liquidity_per_tick_ep, get_observation_ep, get_position_ep,
        get_protocol_fees_ep, get_slot0_ep, get_slot0_unlocked_ep, get_tick_bitmap_ep, get_tick_ep,
        get_tick_spacing_ep, get_token0_ep, get_token1_ep,
    },
};

fn add_entry_points(entry_points: &mut EntryPoints, list: &Vec<EntryPoint>) {
    for e in list {
        entry_points.add_entry_point(e.clone());
    }
}

pub(crate) fn default(is_factory: bool) -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    add_entry_points(&mut entry_points, &timestamp_testing::entry_points());
    add_entry_points(&mut entry_points, &owner::entry_points());
    entry_points.add_entry_point(get_factory_ep());
    entry_points.add_entry_point(get_token0_ep());
    entry_points.add_entry_point(get_token1_ep());
    entry_points.add_entry_point(get_fee_ep());
    entry_points.add_entry_point(get_tick_spacing_ep());
    entry_points.add_entry_point(get_max_liquidity_per_tick_ep());
    entry_points.add_entry_point(get_slot0_ep());
    entry_points.add_entry_point(get_slot0_unlocked_ep());
    entry_points.add_entry_point(get_fee_growth_global0_x128_ep());
    entry_points.add_entry_point(get_fee_growth_global1_x128_ep());
    entry_points.add_entry_point(get_protocol_fees_ep());
    entry_points.add_entry_point(get_liquidity_ep());
    entry_points.add_entry_point(get_tick_ep());
    entry_points.add_entry_point(get_tick_bitmap_ep());
    entry_points.add_entry_point(get_position_ep());
    entry_points.add_entry_point(get_observation_ep());

    if is_factory {
        entry_points.add_entry_point(get_fee_amount_tick_spacing_ep());
        entry_points.add_entry_point(get_pool_map_ep());

        entry_points.add_entry_point(EntryPoint::new(
            String::from("init_factory"),
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));
        entry_points.add_entry_point(EntryPoint::new(
            String::from("get_pool_address"),
            vec![
                Parameter::new("token0", CLType::Key),
                Parameter::new("token1", CLType::Key),
                Parameter::new("fee", CLType::U32),
            ],
            CLType::Key,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            String::from("create_pool"),
            vec![
                Parameter::new("token0", CLType::Key),
                Parameter::new("token1", CLType::Key),
                Parameter::new("fee", CLType::U32),
            ],
            CLType::Key,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));
        entry_points.add_entry_point(EntryPoint::new(
            String::from("enable_fee_amount"),
            vec![
                Parameter::new("fee", CLType::U32),
                Parameter::new("tick_spacing", CLType::I32),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));
    }
    entry_points.add_entry_point(EntryPoint::new(
        String::from("init_pool"),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("init_pool_price"),
        vec![Parameter::new("sqrt_price_x96", CLType::U256)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("snapshot_cumulatives_inside"),
        vec![
            Parameter::new("tick_lower", i32::cl_type()),
            Parameter::new("tick_upper", i32::cl_type()),
        ],
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("observe"),
        vec![Parameter::new(
            "seconds_agos",
            CLType::List(Box::new(u32::cl_type())),
        )],
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("increase_observation_cardinality_next"),
        vec![Parameter::new(
            "observation_cardinality_next",
            u32::cl_type(),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("mint"),
        vec![
            Parameter::new("recipient", CLType::Key),
            Parameter::new("tick_lower", CLType::I32),
            Parameter::new("tick_upper", CLType::I32),
            Parameter::new("amount", CLType::U128),
            Parameter::new("data", CLType::List(Box::new(CLType::U8))),
        ],
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("collect"),
        vec![
            Parameter::new("recipient", CLType::Key),
            Parameter::new("tick_lower", CLType::I32),
            Parameter::new("tick_upper", CLType::I32),
            Parameter::new("amount0_requested", CLType::U128),
            Parameter::new("amount1_requested", CLType::U128),
        ],
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("burn"),
        vec![
            Parameter::new("tick_lower", CLType::I32),
            Parameter::new("tick_upper", CLType::I32),
            Parameter::new("amount", CLType::U128),
        ],
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap"),
        vec![
            Parameter::new("recipient", CLType::Key),
            Parameter::new("zero_for_one", CLType::Bool),
            Parameter::new("amount_specified", CLType::U256),
            Parameter::new("is_amount_specified_positive", CLType::Bool),
            Parameter::new("sqrt_price_limit_x96", CLType::U256),
            Parameter::new("data", CLType::List(Box::new(CLType::U8))),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("flash"),
        vec![
            Parameter::new("recipient", CLType::Key),
            Parameter::new("amount0", CLType::U256),
            Parameter::new("amount1", CLType::U256),
            Parameter::new("data", CLType::List(Box::new(CLType::U8))),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("flash"),
        vec![
            Parameter::new("fee_protocol0", CLType::U8),
            Parameter::new("fee_protocol1", CLType::U8),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("collect_protocol"),
        vec![
            Parameter::new("recipient", CLType::Key),
            Parameter::new("amount0_requested", CLType::U128),
            Parameter::new("amount1_requested", CLType::U128),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("set_fee_protocol"),
        vec![
            Parameter::new("fee_protocol0", u8::cl_type()),
            Parameter::new("fee_protocol1", u8::cl_type()),
        ],
        ProtocolFees::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("get_position_key"),
        vec![
            Parameter::new("tick_lower", CLType::I32),
            Parameter::new("tick_upper", CLType::I32),
            Parameter::new("owner", CLType::Key),
        ],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}
