#![no_std]
#[macro_use]
extern crate alloc;

mod cep47;
pub mod data;
pub mod event;
pub mod periphery;

pub use cep47::{Error, CEP47};
pub use contract_utils;

use alloc::{collections::BTreeMap, string::String};
use casper_types::{Key, U256};
use contract_utils::{ContractContext, OnChainContractStorage};
pub type TokenId = U256;
pub type Meta = BTreeMap<String, String>;

#[derive(Default)]
pub struct NFTToken(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for NFTToken {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl CEP47<OnChainContractStorage> for NFTToken {}
impl NFTToken {
    pub fn constructor(&mut self, name: String, symbol: String, meta: Meta, factory: Key) {
        CEP47::init(self, name, symbol, meta);
        periphery::logics::initialize(factory);
    }
}
