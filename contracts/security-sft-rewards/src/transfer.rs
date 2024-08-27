use concordium_cis2::{Cis2Event, OnReceivingCis2Params, Receiver, TransferEvent};
use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_protocols::concordium_cis2_security::{
    compliance_client, identity_registry_client, CanTransferParam, TokenFrozen, TokenUId,
    TransferredParam,
};
use concordium_protocols::concordium_global_sponsor::{SponsoredParams, SponsoredParamsRaw};
use concordium_std::*;

use super::error::*;
use super::state::State;
use super::types::*;
use crate::state::AddressState;

/// Executes a compliant transfer of token ownership between verified accounts
///
/// This function facilitates the transfer of token ownership from one verified
/// account to another. It can be invoked by the token owner, an authorized
/// operator, or a trusted transaction sponsor.
///
/// # Arguments
///
/// * Implicitly uses `TransferParams` parsed from the parameter cursor
///
/// # Returns
///
/// * `ContractResult<()>` - Indicates the success or failure of the transfer operation
///
/// # Errors
///
/// May return the following errors:
/// * `concordium_std::ParseError` - Failed to parse `TransferParams` from the parameter cursor
/// * `Error::Unauthorized` - The sender lacks the necessary authorization for the transfer
/// * `Error::Custom(CustomContractError::PausedToken)` - The token is currently paused
/// * `Error::InsufficientFunds` - The sender's token balance is insufficient for the transfer
/// * `Error::Custom(CustomContractError::UnVerifiedIdentity)` - The receiver's identity is not
///   verified
/// * `Error::Custom(CustomContractError::InCompliantTransfer)` - The transfer violates compliance
///   rules
/// * `Error::Custom(CustomContractError::LogError)` - Failed to log the transfer event
#[receive(
    contract = "security_sft_rewards",
    name = "transfer",
    enable_logger,
    mutable,
    parameter = "TransferParams",
    error = "Error"
)]
pub fn transfer(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let state = host.state();
    let (sender, params) = {
        let sender = ctx.sender();
        if state.sponsor.is_some_and(|s| sender.matches_contract(&s)) {
            let params: SponsoredParamsRaw = ctx.parameter_cursor().get()?;
            let params: SponsoredParams<TransferParams> = params.try_into()?;
            (Address::Account(params.signer), params.params)
        } else {
            let params: TransferParams = ctx.parameter_cursor().get()?;
            (sender, params)
        }
    };

    let compliance = state.compliance;
    let identity_registry = state.identity_registry;
    let self_address = ctx.self_address();
    let reward_token_range = state.rewards_ids_range;
    let concordium_cis2::TransferParams(transfers) = params;

    for concordium_cis2::Transfer {
        to,
        from,
        amount,
        token_id,
        data,
    } in transfers
    {
        ensure!(amount.gt(&TokenAmount::zero()), Error::InvalidAmount);
        ensure!(
            identity_registry_client::is_verified(host, &identity_registry, &to.address())?,
            Error::UnVerifiedIdentity
        );
        let compliance_token = TokenUId::new(token_id, self_address);
        let compliance_can_transfer =
            compliance_client::can_transfer(host, &compliance, &CanTransferParam {
                token_id: compliance_token,
                to: to.address(),
                amount,
            })?;
        ensure!(compliance_can_transfer, Error::InCompliantTransfer);

        // Transfer token
        let (state, state_builder) = host.state_and_builder();
        let is_paused = state
            .token(&token_id)
            .ok_or(Error::InvalidTokenId)?
            .main()
            .ok_or(Error::InvalidTokenId)?
            .paused;
        ensure!(!is_paused, Error::PausedToken);

        let rewards = {
            let mut from_holder = state.address_mut(&from).ok_or(Error::InvalidAddress)?;
            let from_holder = from_holder.holder_mut().ok_or(Error::InvalidAddress)?;
            let from_holder = from_holder.active_mut().ok_or(Error::RecoveredAddress)?;
            ensure!(
                from.eq(&sender) || from_holder.has_operator(&sender),
                Error::Unauthorized
            );
            from_holder.sub_assign_balance(&token_id, amount)?;
            from_holder.sub_assign_balance_rewards(&reward_token_range, amount)?
        };

        {
            let mut to_holder = state.address_or_insert_holder(&to.address(), state_builder);
            let to_holder = to_holder.holder_mut().ok_or(Error::InvalidAddress)?;
            let to_holder = to_holder.active_mut().ok_or(Error::RecoveredAddress)?;
            to_holder.add_assign_balance(&token_id, amount);
            to_holder.add_assign_balance_rewards(&rewards)?;
        }

        compliance_client::transferred(host, &compliance, &TransferredParam {
            token_id: compliance_token,
            from,
            to: to.address(),
            amount,
        })?;

        logger.log(&Event::Cis2(Cis2Event::Transfer(TransferEvent {
            amount,
            token_id,
            from,
            to: to.address(),
        })))?;

        for (token_id, amount) in rewards {
            logger.log(&Event::Cis2(Cis2Event::Transfer(TransferEvent {
                amount,
                token_id,
                from,
                to: to.address(),
            })))?;
        }

        if let Receiver::Contract(to_contract, entrypoint) = to {
            let parameter = OnReceivingCis2Params {
                token_id,
                amount,
                from,
                data,
            };

            host.invoke_contract(
                &to_contract,
                &parameter,
                entrypoint.as_entrypoint_name(),
                Amount::zero(),
            )?;
        }
    }

    Ok(())
}

/// Forces the transfer of a specific amount of tokens from one verified account
/// to another verified. This function can be called by a trusted agent.
/// This function can be used to transfer tokens that are not compliant.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating the success or failure of the
/// operation.
///
/// # Errors
///
/// This method will return an error if:
/// * `concordium_std::ParseError` - The parameter cursor cannot parse the `TransferParams`.
/// * `Error::Unauthorized` - The sender is not authorized to perform the transfer. Sender is not an agent.
/// * `Error::Custom(CustomContractError::PausedToken)` - The token is paused.
/// * `Error::InsufficientFunds` - The sender does not have enough tokens.
/// * `Error::Custom(CustomContractError::UnVerifiedIdentity)` - The receiver's identity is not verified.
/// * `Error::Custom(CustomContractError::LogError)` - The logger failed to log the event.
#[receive(
    contract = "security_sft_rewards",
    name = "forcedTransfer",
    enable_logger,
    mutable,
    parameter = "TransferParams",
    error = "Error"
)]
pub fn forced_transfer(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let state = host.state();
    let is_authorized =
        state
            .address(&ctx.sender())
            .is_some_and(|a: StateRef<AddressState<ExternStateApi>>| {
                a.is_agent(&[AgentRole::ForcedTransfer])
            });
    ensure!(is_authorized, Error::Unauthorized);

    let compliance = state.compliance;
    let identity_registry = state.identity_registry;
    let self_address = ctx.self_address();
    let reward_token_range = state.rewards_ids_range;
    let params: TransferParams = ctx.parameter_cursor().get()?;

    for concordium_cis2::Transfer {
        to,
        from,
        amount,
        token_id,
        data,
    } in params.0
    {
        ensure!(amount.gt(&TokenAmount::zero()), Error::InvalidAmount);
        ensure!(
            identity_registry_client::is_verified(host, &identity_registry, &to.address())?,
            Error::UnVerifiedIdentity
        );
        ensure!(amount.gt(&TokenAmount::zero()), Error::InvalidAmount);

        let compliance_token = TokenUId::new(token_id, self_address);

        // Transfer token
        let (state, state_builder) = host.state_and_builder();
        let is_paused = state
            .token(&token_id)
            .ok_or(Error::InvalidTokenId)?
            .main()
            .ok_or(Error::InvalidTokenId)?
            .paused;
        ensure!(!is_paused, Error::PausedToken);

        let (rewards, un_frozen_amount) = {
            let mut from_holder = state.address_mut(&from).ok_or(Error::InvalidAddress)?;
            let from_holder = from_holder.holder_mut().ok_or(Error::InvalidAddress)?;
            let from_holder = from_holder.active_mut().ok_or(Error::RecoveredAddress)?;
            let un_frozen_amount = from_holder.un_freeze_balance_to_match(&token_id, amount)?;
            from_holder.sub_assign_balance(&token_id, amount)?;
            let rewards = from_holder.sub_assign_balance_rewards(&reward_token_range, amount)?;

            (rewards, un_frozen_amount)
        };

        {
            let mut to_holder = state.address_or_insert_holder(&to.address(), state_builder);
            let to_holder = to_holder.holder_mut().ok_or(Error::InvalidAddress)?;
            let to_holder = to_holder.active_mut().ok_or(Error::RecoveredAddress)?;
            to_holder.add_assign_balance(&token_id, amount);
            to_holder.add_assign_balance_rewards(&rewards)?;
        }

        compliance_client::transferred(host, &compliance, &TransferredParam {
            token_id: compliance_token,
            from,
            to: to.address(),
            amount,
        })?;

        if un_frozen_amount.gt(&TokenAmount::zero()) {
            logger.log(&Event::TokenUnFrozen(TokenFrozen {
                token_id,
                amount: un_frozen_amount,
                address: from,
            }))?;
        }
        logger.log(&Event::Cis2(Cis2Event::Transfer(TransferEvent {
            amount,
            token_id,
            from,
            to: to.address(),
        })))?;
        for (token_id, amount) in rewards {
            logger.log(&Event::Cis2(Cis2Event::Transfer(TransferEvent {
                amount,
                token_id,
                from,
                to: to.address(),
            })))?;
        }

        if let Address::Contract(_from_contract) = from {
            // TODO: there should be a way to notify that the transfer has been
            // forced Ex. A token is sent to the marketplace for
            // selling. Upon a forced transfer since marketplace
            // would not know that the it does not have the token authority
            // would continue to sell the token. Without anyone being able to
            // buy it.
        }

        if let Receiver::Contract(to_contract, entrypoint) = to {
            let parameter = OnReceivingCis2Params {
                token_id,
                amount,
                from,
                data,
            };

            host.invoke_contract(
                &to_contract,
                &parameter,
                entrypoint.as_entrypoint_name(),
                Amount::zero(),
            )?;
        }
    }

    Ok(())
}
