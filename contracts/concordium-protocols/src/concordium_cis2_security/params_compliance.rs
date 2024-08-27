use concordium_cis2::{IsTokenAmount, IsTokenId};
use concordium_std::{Address, SchemaType, Serialize};

use super::TokenUId;

/// Parameters for the `can_transfer` function.
#[derive(Serialize, SchemaType)]
pub struct CanTransferParam<T: IsTokenId, A: IsTokenAmount> {
    /// The ID of the token to transfer.
    pub token_id: TokenUId<T>,
    /// The address to transfer to.
    pub to:       Address,
    /// The amount of tokens to transfer.
    pub amount:   A,
}

/// Parameters for the `burned` function.
#[derive(Serialize, SchemaType)]
pub struct BurnedParam<T: IsTokenId, A: IsTokenAmount> {
    /// The ID of the token that was burned.
    pub token_id: TokenUId<T>,
    /// The address of the owner of the burned tokens.
    pub owner:    Address,
    /// The amount of tokens that were burned.
    pub amount:   A,
}

/// Parameters for the `minted` function.
#[derive(Serialize, SchemaType)]
pub struct MintedParam<T: IsTokenId, A: IsTokenAmount> {
    /// The ID of the token that was minted.
    pub token_id: TokenUId<T>,
    /// The address of the owner of the minted tokens.
    pub owner:    Address,
    /// The amount of tokens that were minted.
    pub amount:   A,
}

/// Parameters for the `transferred` function.
#[derive(Serialize, SchemaType)]
pub struct TransferredParam<T: IsTokenId, A: IsTokenAmount> {
    /// The ID of the token that was transferred.
    pub token_id: TokenUId<T>,
    /// The address of the sender of the transfer.
    pub from:     Address,
    /// The address of the receiver of the transfer.
    pub to:       Address,
    /// The amount of tokens that were transferred.
    pub amount:   A,
}
