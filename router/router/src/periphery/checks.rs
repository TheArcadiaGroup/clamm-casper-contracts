use casper_types::U256;
use common::error::require;
use contract_utilities::helpers::{current_block_timestamp, get_immediate_caller_key};

use crate::{NFTToken, CEP47};

pub fn check_deadline(deadline: u64) {
    require(
        current_block_timestamp() <= deadline,
        common::error::Error::ErrTransactionTooOld,
    );
}

pub fn is_authorized_for_token(token_id: &U256) {
    require(
        NFTToken::default().is_authorized(*token_id, get_immediate_caller_key()),
        common::error::Error::ErrNotApproved,
    );
}
