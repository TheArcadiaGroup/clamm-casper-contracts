use casper_event_standard::Schemas;
use common::router_events::*;
pub fn event_schemas() -> Schemas {
    Schemas::new()
        .with::<IncreaseLiquidity>()
        .with::<DecreaseLiquidity>()
        .with::<Collect>()
}

pub fn init_events() {
    casper_event_standard::init(event_schemas());
}
