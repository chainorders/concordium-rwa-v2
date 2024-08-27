use concordium_cis2::{Cis2Event, OnReceivingCis2Params, Receiver, TransferEvent};
use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_protocols::concordium_cis2_security::{
    compliance_client, identity_registry_client, CanTransferParam, TokenFrozen, TokenUId,
    TransferredParam,
};
use concordium_protocols::concordium_global_sponsor::{SponsoredParams, SponsoredParamsRaw};
use concordium_rwa_utils::state_implementations::agent_with_roles_state::IAgentWithRolesState;
use concordium_rwa_utils::state_implementations::cis2_security_state::ICis2SecurityState;
use concordium_rwa_utils::state_implementations::holders_state::IHoldersState;
use concordium_rwa_utils::state_implementations::rewards_state::IRewardsState;
use concordium_rwa_utils::state_implementations::sponsors_state::ISponsorsState;
use concordium_std::*;

use super::error::*;
use super::state::State;
use super::types::*;

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
        if state.is_sponsor(&sender) {
            let params: SponsoredParamsRaw = ctx.parameter_cursor().get()?;
            let params: SponsoredParams<TransferParams> = params.try_into()?;
            (Address::Account(params.signer), params.params)
        } else {
            let params: TransferParams = ctx.parameter_cursor().get()?;
            (sender, params)
        }
    };

    let concordium_cis2::TransferParams(transfers) = params;
    let compliance = state.compliance();

    for concordium_cis2::Transfer {
        to,
        from,
        amount,
        token_id,
        data,
    } in transfers
    {
        let compliance_token = TokenUId::new(token_id, ctx.self_address());
        let state = host.state();
        ensure!(
            identity_registry_client::is_verified(host, state.identity_registry(), &to.address())?,
            Error::UnVerifiedIdentity
        );
        let compliance_can_transfer =
            compliance_client::can_transfer(host, state.compliance(), &CanTransferParam {
                token_id: compliance_token,
                to: to.address(),
                amount,
            })?;
        ensure!(compliance_can_transfer, Error::InCompliantTransfer);
        ensure!(
            from.eq(&sender) || state.is_operator(&from, &sender),
            Error::Unauthorized
        );

        // Transfer token
        let (state, state_builder) = host.state_and_builder();
        ensure!(token_id.eq(&state.tracked_token_id), Error::InvalidTokenId);
        state.transfer(
            &from,
            &to.address(),
            &token_id,
            &amount,
            false,
            state_builder,
        )?;
        // transfer attached rewards
        let transferred_rewards =
            state.transfer_rewards(&from, &to.address(), &amount, state_builder)?;

        compliance_client::transferred(host, compliance, &TransferredParam {
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
        for transfer in transferred_rewards
            .iter()
            .filter(|r| r.token_amount.gt(&TokenAmount::zero()))
        {
            logger.log(&Event::Cis2(Cis2Event::Transfer(TransferEvent {
                amount: transfer.token_amount,
                token_id: transfer.token_id,
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
    let concordium_cis2::TransferParams(transfers) = ctx.parameter_cursor().get()?;
    let state = host.state();
    ensure!(
        state.is_agent(&ctx.sender(), vec![AgentRole::ForcedTransfer]),
        Error::Unauthorized
    );

    for concordium_cis2::Transfer {
        to,
        from,
        amount,
        token_id,
        data,
    } in transfers
    {
        let state = host.state();
        ensure!(
            identity_registry_client::is_verified(host, state.identity_registry(), &to.address())?,
            Error::UnVerifiedIdentity
        );

        let (state, state_builder) = host.state_and_builder();
        let un_frozen_balance = state.transfer(
            &from,
            &to.address(),
            &token_id,
            &amount,
            true,
            state_builder,
        )?;
        let transferred_rewards =
            state.transfer_rewards(&from, &to.address(), &amount, state_builder)?;
        // Adjust the frozen balance of the sender.
        compliance_client::transferred(host, host.state().compliance(), &TransferredParam {
            token_id: TokenUId::new(token_id, ctx.self_address()),
            from,
            to: to.address(),
            amount,
        })?;

        logger.log(&Event::TokenUnFrozen(TokenFrozen {
            token_id,
            amount: un_frozen_balance,
            address: from,
        }))?;
        logger.log(&Event::Cis2(Cis2Event::Transfer(TransferEvent {
            amount,
            token_id,
            from,
            to: to.address(),
        })))?;
        for transfer in transferred_rewards
            .iter()
            .filter(|r| r.token_amount.gt(&TokenAmount::zero()))
        {
            logger.log(&Event::Cis2(Cis2Event::Transfer(TransferEvent {
                amount: transfer.token_amount,
                token_id: transfer.token_id,
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
