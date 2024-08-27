use concordium_cis2::{AdditionalData, Cis2Client, OnReceivingCis2Params, Receiver, Transfer};
use concordium_rwa_utils::state_implementations::token_deposits_state::IDepositedTokensState;
use concordium_std::*;

use super::error::*;
use super::event::{Event, TokenDeposited};
use super::exchange::{exchange_internal, ExchangeParams};
use super::list::{list_internal, ListParams};
use super::state::State;
use super::types::{Cis2TokenAmount, ContractResult, TokenId, TokenOwnerUId, TokenUId};

pub type DepositParams = OnReceivingCis2Params<TokenId, Cis2TokenAmount>;

#[receive(
    contract = "rwa_market",
    name = "deposit",
    mutable,
    parameter = "DepositParams",
    error = "super::error::Error",
    enable_logger
)]
pub fn deposit(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let sender = match ctx.sender() {
        Address::Account(_) => bail!(Error::Unauthorized),
        Address::Contract(sender) => sender,
    };

    let params: DepositParams = ctx.parameter_cursor().get()?;
    let from = match params.from {
        Address::Account(address) => address,
        Address::Contract(_) => bail!(Error::OnlyAccount),
    };

    let deposited_token_uid = TokenOwnerUId {
        token_id: TokenUId {
            contract: sender,
            id:       params.token_id,
        },
        owner:    Receiver::Account(from),
    };
    host.state_mut()
        .inc_deposits(deposited_token_uid.clone(), params.amount);
    logger.log(&Event::Deposited(TokenDeposited {
        token_id: deposited_token_uid.clone().token_id,
        owner:    from,
        amount:   params.amount,
    }))?;

    if params.data.size().ne(&0u32) {
        let mut cursor = Cursor::new(params.data.to_owned());
        let data: Result<ListParams, ParseError> = ListParams::deserial(&mut cursor);
        if let Ok(data) = data {
            ensure!(
                params.from.matches_account(&data.owner),
                Error::Unauthorized
            );
            ensure!(
                deposited_token_uid.matches_token(&data.token_id),
                Error::InvalidDepositData
            );
            list_internal(data, host, logger)?
        } else {
            cursor.offset = 0;
            let data = ExchangeParams::deserial(&mut cursor);
            if let Ok(data) = data {
                ensure!(
                    params.from.matches_account(&data.payer),
                    Error::Unauthorized
                );
                ensure!(
                    host.state().can_be_paid_by(&deposited_token_uid.token_id),
                    Error::InvalidDepositData
                );
                exchange_internal(
                    ctx.self_address(),
                    ctx.owner(),
                    data,
                    Amount::zero(),
                    host,
                    logger,
                )?
            }
        }
    }

    Ok(())
}

#[derive(Serialize, SchemaType, Clone)]
pub struct WithdrawParams {
    pub token_id: TokenUId,
    pub owner:    AccountAddress,
    pub amount:   Cis2TokenAmount,
}

#[receive(
    contract = "rwa_market",
    name = "withdraw",
    mutable,
    parameter = "WithdrawParams",
    error = "super::error::Error",
    enable_logger
)]
pub fn withdraw(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let params: WithdrawParams = ctx.parameter_cursor().get()?;
    ensure!(
        ctx.sender().matches_account(&params.owner),
        Error::Unauthorized
    );
    let token_owner_uid = params.token_id.to_token_owner_uid(params.owner.into());

    host.state_mut()
        .dec_deposits(&token_owner_uid, params.amount)?;
    Cis2Client::new(params.token_id.contract)
        .transfer::<_, _, _, ()>(host, Transfer {
            amount:   params.amount,
            from:     Address::Contract(ctx.self_address()),
            to:       token_owner_uid.owner,
            token_id: token_owner_uid.token_id.id.to_owned(),
            data:     AdditionalData::empty(),
        })
        .map_err(|_| Error::Cis2WithdrawError)?;
    logger.log(&Event::Withdraw(TokenDeposited {
        token_id: params.token_id,
        owner:    params.owner,
        amount:   params.amount,
    }))?;

    Ok(())
}

#[derive(Serialize, SchemaType)]
pub struct BalanceOfDepositParams {
    pub token_id: TokenUId,
    pub address:  AccountAddress,
}

#[receive(
    contract = "rwa_market",
    name = "balanceOfDeposited",
    parameter = "BalanceOfDepositParams",
    error = "super::error::Error",
    return_value = "Cis2TokenAmount"
)]
pub fn balance_of_deposited(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<Cis2TokenAmount> {
    let params: BalanceOfDepositParams = ctx.parameter_cursor().get()?;
    Ok(host
        .state()
        .balance_of_deposited(&params.token_id.to_token_owner_uid(params.address.into())))
}
