use casper_contract::contract_api::runtime;

pub fn log(_msg: &str) {
    runtime::print(_msg);
}
