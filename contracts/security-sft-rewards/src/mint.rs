use concordium_cis2::{Cis2Event, MintEvent};
use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_protocols::concordium_cis2_security::{
    compliance_client, identity_registry_client, CanTransferParam, MintedParam, TokenUId,
};
use concordium_std::*;

use super::error::*;
use super::state::State;
use super::types::*;

#[receive(
    contract = "security_sft_rewards",
    name = "mint",
    enable_logger,
    mutable,
    parameter = "MintParams",
    error = "Error"
)]
pub fn mint(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let self_address = ctx.self_address();
    let params: MintParams = ctx.parameter_cursor().get()?;

    let state = host.state();
    let is_authorized = state
        .address(&ctx.sender())
        .is_some_and(|a| a.is_agent(&[AgentRole::Mint]));
    ensure!(is_authorized, Error::Unauthorized);

    let compliance = state.compliance;
    let identity_registry_client = state.identity_registry;
    let (_, max_reward_token_id) = state.rewards_ids_range;

    for MintParam {
        address: owner,
        amount,
    } in params.owners
    {
        let owner = Address::Account(owner);
        ensure!(amount.gt(&TokenAmount::zero()), Error::InvalidAmount);
        ensure!(
            identity_registry_client::is_verified(host, &identity_registry_client, &owner)?,
            Error::UnVerifiedIdentity
        );
        let compliance_token = TokenUId::new(params.token_id, self_address);
        let compliance_can_transfer =
            compliance_client::can_transfer(host, &compliance, &CanTransferParam {
                token_id: compliance_token,
                amount,
                to: owner,
            })?;
        ensure!(compliance_can_transfer, Error::InCompliantTransfer);

        let (state, state_builder) = host.state_and_builder();
        {
            // Mint tokens
            let mut address = state.address_or_insert_holder(&owner, state_builder);
            let holder = address.holder_mut().ok_or(Error::InvalidAddress)?;
            let active_holder = holder.active_mut().ok_or(Error::RecoveredAddress)?;
            active_holder.add_assign_balance(&params.token_id, amount);
            active_holder.add_assign_balance(&max_reward_token_id, amount);
        }
        {
            // Update minted supply
            state
                .token_mut(&params.token_id)
                .ok_or(Error::InvalidTokenId)?
                .main_mut()
                .ok_or(Error::InvalidTokenId)?
                .add_assign_supply(amount)?;
            state
                .token_mut(&max_reward_token_id)
                .ok_or(Error::InvalidTokenId)?
                .reward_mut()
                .ok_or(Error::InvalidTokenId)?
                .add_assign_supply(amount);
        }

        compliance_client::minted(host, &compliance, &MintedParam {
            token_id: TokenUId::new(params.token_id, self_address),
            amount,
            owner,
        })?;

        logger.log(&Event::Cis2(Cis2Event::Mint(MintEvent {
            token_id: params.token_id,
            amount,
            owner,
        })))?;
        logger.log(&Event::Cis2(Cis2Event::Mint(MintEvent {
            token_id: max_reward_token_id,
            amount,
            owner,
        })))?;
    }

    Ok(())
}
