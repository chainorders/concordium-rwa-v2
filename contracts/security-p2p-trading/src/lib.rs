use concordium_cis2::{
    AdditionalData, OnReceivingCis2DataParams, Receiver, TokenAmountU64, TokenIdVec, Transfer,
};
use concordium_protocols::concordium_cis2_ext::cis2_client::{self, Cis2ClientError};
use concordium_protocols::concordium_cis2_security::TokenUId;
use concordium_protocols::rate::Rate;
use concordium_rwa_utils::conversions::to_additional_data;
use concordium_std::ops::SubAssign;
use concordium_std::*;

pub type TokenAmount = TokenAmountU64;
pub type AnyTokenUId = TokenUId<TokenIdVec>;

/// Represents Sell event.
/// This is the event that is emitted when a user deposits tokens to be sold.
#[derive(Serialize, SchemaType)]
pub struct SellEvent {
    /// The address of the user who deposited the tokens.
    pub from:   AccountAddress,
    /// The amount of tokens that were deposited.
    pub amount: TokenAmount,
    /// The rate at which a buyer can convert deposited tokens to currency token.
    pub rate:   Rate,
}

/// Represents SellCancelled event.
/// This is the event that is emitted when a user cancels a sell position.
#[derive(Serialize, SchemaType)]
pub struct SellCancelledEvent {
    pub from:   AccountAddress,
    pub amount: TokenAmount,
}

/// Represents Exchange event.
/// This event is emitted when a user exchanges tokens with another user.
#[derive(Serialize, SchemaType)]
pub struct Exchange {
    pub payer:       AccountAddress,
    pub pay_amount:  TokenAmount,
    pub sell_amount: TokenAmount,
    pub seller:      AccountAddress,
}

#[derive(Serialize, SchemaType)]
pub enum Event {
    Initialized(InitParam),
    Sell(SellEvent),
    SellCancelled(SellCancelledEvent),
    Exchange(Exchange),
}

#[derive(Serial, Reject, SchemaType)]
pub enum Error {
    ParseError,
    Unauthorized,
    Cis2CallError,
    InvalidToken,
    SellPositionExists,
    SellPositionMissing,
    InvalidConversion,
    InvalidAmount,
    LogError,
}
impl From<ParseError> for Error {
    fn from(_: ParseError) -> Self { Error::ParseError }
}
impl From<Cis2ClientError> for Error {
    fn from(_: Cis2ClientError) -> Self { Error::Cis2CallError }
}
impl From<LogError> for Error {
    fn from(_: LogError) -> Self { Error::LogError }
}
pub type ContractResult<T> = Result<T, Error>;

#[derive(Serialize, Clone, SchemaType, PartialEq, Debug)]
pub struct Deposit {
    pub amount: TokenAmount,
    pub rate:   Rate,
}

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S=StateApi> {
    token:    AnyTokenUId,
    currency: AnyTokenUId,
    deposits: StateMap<AccountAddress, Deposit, S>,
}

/// Initialization parameters for the contract.
#[derive(Serialize, SchemaType, Clone)]
pub struct InitParam {
    /// The token that is being sold.
    pub token:    AnyTokenUId,
    /// The token that is being used to pay for the tokens being sold.
    pub currency: AnyTokenUId,
}

#[init(
    contract = "security_p2p_trading",
    event = "Event",
    error = "Error",
    parameter = "InitParam",
    enable_logger
)]
pub fn init(
    ctx: &InitContext,
    state_builder: &mut StateBuilder,
    logger: &mut Logger,
) -> InitResult<State> {
    let params: InitParam = ctx.parameter_cursor().get()?;
    logger.log(&Event::Initialized(params.clone()))?;
    Ok(State {
        token:    params.token,
        currency: params.currency,
        deposits: state_builder.new_map(),
    })
}

/// Represents a sell position.
#[derive(Serialize, SchemaType)]
pub struct TransferSellParams {
    /// The amount of tokens that are being sold.
    pub amount: TokenAmount,
    /// The rate at which a buyer can convert deposited tokens to currency token.
    pub rate:   Rate,
}

/// This function is called when a user wants to sell tokens.
/// To be able to call this function the user must have added current contract as an operator in the token contract for the token which is to be sold.
/// The function will transfer the tokens to be sold from the user to the current contract.
/// Tokens are received in the `sell` entrypoint.
#[receive(
    contract = "security_p2p_trading",
    name = "transferSell",
    mutable,
    parameter = "TransferSellParams",
    error = "Error"
)]
pub fn transfer_sell(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    let params: TransferSellParams = ctx.parameter_cursor().get()?;
    let token = host.state().token.clone();
    cis2_client::transfer_single(host, &token.contract, Transfer {
        token_id: token.id,
        amount:   params.amount,
        from:     ctx.sender(),
        to:       Receiver::from_contract(
            ctx.self_address(),
            OwnedEntrypointName::new_unchecked("sell".into()),
        ),
        data:     to_additional_data(params.rate).map_err(|_| Error::ParseError)?,
    })?;
    Ok(())
}

pub type SellReceiveParams = OnReceivingCis2DataParams<TokenIdVec, TokenAmount, Rate>;

/// This function is called when a user wants to sell tokens.
/// This function can only be called by a smart contract.
/// In order to call this function the token holder should transfer the tokens to be sold to the current contract and token contract should call this function.
/// Current contract will only allow trusted token contracts to call this function.
#[receive(
    contract = "security_p2p_trading",
    name = "sell",
    mutable,
    parameter = "SellReceiveParams",
    error = "Error",
    enable_logger
)]
pub fn sell(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let params: SellReceiveParams = ctx.parameter_cursor().get()?;
    let state = host.state();

    let incoming_token = TokenUId {
        id:       params.token_id.clone(),
        contract: match ctx.sender() {
            Address::Account(_) => bail!(Error::Unauthorized),
            Address::Contract(c) => c,
        },
    };
    ensure!(incoming_token.eq(&state.token), Error::InvalidToken);
    let from = match params.from {
        Address::Account(from) => from,
        Address::Contract(_) => bail!(Error::Unauthorized),
    };
    host.state_mut()
        .deposits
        .entry(from)
        .vacant_or(Error::SellPositionExists)?
        .insert(Deposit {
            amount: params.amount,
            rate:   params.data,
        });
    logger.log(&Event::Sell(SellEvent {
        from,
        amount: params.amount,
        rate: params.data,
    }))?;
    Ok(())
}

/// This function should be called by the owner of the tokens being sold to cancel the sell position.
/// The function will transfer the tokens back to the user.
#[receive(
    contract = "security_p2p_trading",
    name = "cancelSell",
    mutable,
    error = "Error",
    enable_logger
)]
pub fn cancel_sell(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let sender = match ctx.sender() {
        Address::Account(from) => from,
        Address::Contract(_) => bail!(Error::Unauthorized),
    };
    let deposit_token = host.state().token.clone();
    let deposits = host
        .state_mut()
        .deposits
        .remove_and_get(&sender)
        .ok_or(Error::SellPositionMissing)?;
    cis2_client::transfer_single(host, &deposit_token.contract, Transfer {
        token_id: deposit_token.id,
        amount:   deposits.amount,
        from:     ctx.self_address().into(),
        to:       Receiver::Account(sender),
        data:     AdditionalData::empty(),
    })?;
    logger.log(&Event::SellCancelled(SellCancelledEvent {
        from:   sender,
        amount: deposits.amount,
    }))?;
    Ok(())
}

#[derive(Serialize, SchemaType)]
pub struct ForceCancelSellParams {
    pub from: AccountAddress,
    pub to:   AccountAddress,
}

/// This function can only be called by the owner of the current contract.
/// The function will transfer the tokens to the user specified in the `from` field to the user specified in the `to` field.
/// The function will also remove the sell position of the user specified in the `from` field.
/// This functions is intended to be called in cases where the security token holder has been recovered in the token contract but the sell position is still active.
#[receive(
    contract = "security_p2p_trading",
    name = "forceCancelSell",
    mutable,
    parameter = "ForceCancelSellParams",
    error = "Error",
    enable_logger
)]
pub fn force_cancel_sell(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        Error::Unauthorized
    );
    let ForceCancelSellParams { from, to } = ctx.parameter_cursor().get()?;
    let deposit_token = host.state().token.clone();
    let deposits = host
        .state_mut()
        .deposits
        .remove_and_get(&from)
        .ok_or(Error::SellPositionMissing)?;
    cis2_client::transfer_single(host, &deposit_token.contract, Transfer {
        token_id: deposit_token.id,
        amount:   deposits.amount,
        from:     ctx.self_address().into(),
        to:       Receiver::Account(to),
        data:     AdditionalData::empty(),
    })?;
    logger.log(&Event::SellCancelled(SellCancelledEvent {
        from,
        amount: deposits.amount,
    }))?;
    Ok(())
}

#[derive(Serialize, SchemaType)]
pub struct TransferExchangeParams {
    pub pay: TokenAmount,
    pub get: ExchangeParams,
}

#[derive(Serialize, SchemaType)]
pub struct ExchangeParams {
    pub from: AccountAddress,
    pub rate: Rate,
}

/// This function should be called by the buyer of the tracked security token.
/// The function will transfer currency token from the sender to the contract. The transfer is received by the `exchange` entrypoint.
/// To be able to call this function the sender must have the current contract added as an operator to the currency token contract.
#[receive(
    contract = "security_p2p_trading",
    name = "transferExchange",
    mutable,
    parameter = "TransferExchangeParams",
    error = "Error"
)]
pub fn transfer_exchange(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    let params: TransferExchangeParams = ctx.parameter_cursor().get()?;
    let currency = host.state().currency.clone();
    cis2_client::transfer_single(host, &currency.contract, Transfer {
        token_id: currency.id,
        amount:   params.pay,
        from:     ctx.sender(),
        to:       Receiver::from_contract(
            ctx.self_address(),
            OwnedEntrypointName::new_unchecked("exchange".into()),
        ),
        data:     to_additional_data(params.get).map_err(|_| Error::ParseError)?,
    })?;
    Ok(())
}

pub type ExchangeReceiveParams = OnReceivingCis2DataParams<TokenIdVec, TokenAmount, ExchangeParams>;

/// This function is called by the currency token contract when the currency token is received.
/// This would happen when the currency token holder transfers currency token to the current contract.
/// The function will calculate the amounts of the security token to be transferred to the buyer and the currency token to be transferred to the seller
/// and will settle the amounts.
#[receive(
    contract = "security_p2p_trading",
    name = "exchange",
    mutable,
    parameter = "ExchangeReceiveParams",
    error = "Error",
    enable_logger
)]
pub fn exchange(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let ExchangeReceiveParams {
        amount: pay_amount,
        from: payer,
        token_id: currency_token_id,
        data: ExchangeParams { from: seller, rate },
    } = ctx.parameter_cursor().get()?;
    let currency = TokenUId {
        id:       currency_token_id,
        contract: match ctx.sender() {
            Address::Account(_) => bail!(Error::Unauthorized),
            Address::Contract(c) => c,
        },
    };
    let payer = match payer {
        Address::Account(payer) => payer,
        Address::Contract(_) => bail!(Error::Unauthorized),
    };

    let (token, pay_amount, sell_amount) = {
        let state = host.state_mut();
        ensure!(currency.eq(&state.currency), Error::InvalidToken);

        let mut deposit = state
            .deposits
            .get_mut(&seller)
            .ok_or(Error::SellPositionMissing)?;
        ensure!(deposit.rate.eq(&rate), Error::SellPositionMissing);

        let (sell_amount, un_converted_pay_amount) = deposit
            .rate
            .convert(&pay_amount.0)
            .map_err(|_| Error::InvalidConversion)?;

        let sell_amount = TokenAmountU64(sell_amount);
        ensure_eq!(un_converted_pay_amount, 0, Error::InvalidAmount);
        ensure!(sell_amount.le(&deposit.amount), Error::InvalidAmount);
        ensure!(sell_amount.gt(&TokenAmountU64(0)), Error::InvalidAmount);
        deposit.amount.sub_assign(sell_amount);

        (state.token.clone(), pay_amount, sell_amount)
    };

    cis2_client::transfer_single(host, &currency.contract, Transfer {
        token_id: currency.id.clone(),
        amount:   pay_amount,
        from:     ctx.self_address().into(),
        to:       Receiver::Account(seller),
        data:     AdditionalData::empty(),
    })?;
    cis2_client::transfer_single(host, &token.contract, Transfer {
        token_id: token.id,
        amount:   sell_amount,
        from:     ctx.self_address().into(),
        to:       Receiver::Account(payer),
        data:     AdditionalData::empty(),
    })?;
    logger.log(&Event::Exchange(Exchange {
        payer,
        pay_amount,
        sell_amount,
        seller,
    }))?;

    Ok(())
}

#[derive(Serialize, SchemaType)]
pub struct GetDepositParams {
    pub from: AccountAddress,
}

/// This function is non mutable and returns the deposit of the seller specified by the `from` parameter.
#[receive(
    contract = "security_p2p_trading",
    name = "getDeposit",
    parameter = "GetDepositParams",
    return_value = "Deposit"
)]
pub fn get_deposit(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<Deposit> {
    let params: GetDepositParams = ctx.parameter_cursor().get()?;
    let deposit = host
        .state()
        .deposits
        .get(&params.from)
        .ok_or(Error::SellPositionMissing)?;

    Ok(deposit.clone())
}
