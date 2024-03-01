use casper_event_standard::Schemas;
use common::pool_events::*;
pub fn event_schemas() -> Schemas {
    Schemas::new()
        .with::<Initialize>()
        .with::<Mint>()
        .with::<Collect>()
        .with::<Burn>()
        .with::<Swap>()
        .with::<Flash>()
        .with::<IncreaseObservationCardinalityNext>()
        .with::<SetFeeProtocol>()
        .with::<CollectProtocol>()
        .with::<PoolCreated>()
        .with::<SnapshotCumulativesInside>()
}

pub fn init_events() {
    casper_event_standard::init(event_schemas());
}
