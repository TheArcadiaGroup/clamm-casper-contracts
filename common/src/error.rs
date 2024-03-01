use casper_types::ApiError;
use contract_utilities::helpers;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Error {
    ErrMoreThan = 15000,
    ContractLocked,
    OnlyOwnerCanRevoke,
    ContractPaused,
    InvalidTestingMode,
    InvalidMulDiv,
    ErrLS,
    ErrLA,
    ErrT,
    ErrR,
    InvalidNumber,
    SqrtRatioInternalError,
    ConvertU160Overflow,
    ContractAlreadyInitialized,
    FailedToCreateDictionary,
    InvalidFactoryOwner,
    ErrLO,
    ErrFlipTick,
    ErrTLU,
    ErrTLM,
    ErrTUM,
    ErrLowerUninitialized,
    ErrUpperUninitialized,
    ErrNP,
    ErrI,
    ErrOld,
    ErrMintAmount,
    ErrMintM0,
    ErrMintM1,
    ErrSwapAS,
    ErrSwapLOK,
    ErrSwapSPL,
    ErrIIA,
    ErrL,
    ErrF0,
    ErrF1,
    ErrFeeProtocol,
    ErrNotPool,
    ErrSameToken,
    ErrTokenNull,
    ErrTickSpacingNull,
    ErrPoolExist,
    ErrAI,
    ErrFactoryFee,
    ErrInvalidTickSpacing,
    ErrFeeExist,
    ErrSwapCallee,
    ErrInvalidTestingMode,
    ErrLOK,
    ErrTransactionTooOld,
    ErrPriceSlippageCheck,
    ErrInvalidMintCallback,
    ErrNotApproved,
    ErrInvalidLiquidity,
    ErrInvalidCollectParams,
    ErrInvalidSwapCallbackParams,
    ErrPositionNotCleared,
    ErrInvalidAmountOut,
    ErrTooMuchRequested,
    ErrTooLittleReceived,
    ErrInvalidTokenOrder,
    ErrInvalidLiquiditySessionParams,
    ErrInsufficientBalanceWCSPR,
}

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        ApiError::User(e as u16)
    }
}

pub fn as_u16(err: Error) -> u16 {
    err as u16
}

pub fn require(v: bool, e: Error) {
    if !v {
        helpers::require(v, e.into());
    }
}
