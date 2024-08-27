use concordium_std::*;

use super::types::*;

#[derive(Serialize, SchemaType, Debug)]
pub struct TokenDeposited {
    pub token_id: TokenUId,
    pub owner:    AccountAddress,
    pub amount:   Cis2TokenAmount,
}

#[derive(Serialize, SchemaType, Debug)]
pub struct TokenListed {
    pub token_id: TokenUId,
    pub owner:    AccountAddress,
    pub supply:   Cis2TokenAmount,
}

#[derive(Serialize, SchemaType, Debug)]
pub struct TokenDeListed {
    pub token_id: TokenUId,
    pub owner:    AccountAddress,
}

#[derive(Serialize, SchemaType, Debug, Eq, PartialEq, Clone)]
pub enum PaymentTokenUId {
    Cis2(TokenUId),
    CCD,
}

#[derive(Serialize, SchemaType, Debug, Eq, PartialEq, Clone, Copy)]
pub enum PaymentAmount {
    Cis2(Cis2TokenAmount),
    CCD(Amount),
}

#[derive(Serialize, SchemaType, Debug)]
pub struct TokenExchanged {
    // Token which was to be exchanged
    pub buy_token_id:      TokenUId,
    pub buy_amount:        Cis2TokenAmount,
    pub buy_token_owner:   AccountAddress,
    // token by which the input token was exchanged
    pub pay_token_id:      PaymentTokenUId,
    pub pay_amount:        PaymentAmount,
    pub pay_token_owner:   AccountAddress,
    pub commission_amount: PaymentAmount,
}

#[derive(Serialize, SchemaType, Debug)]
pub enum Event {
    Deposited(TokenDeposited),
    Withdraw(TokenDeposited),
    Listed(TokenListed),
    DeListed(TokenDeListed),
    Exchanged(TokenExchanged),
}
