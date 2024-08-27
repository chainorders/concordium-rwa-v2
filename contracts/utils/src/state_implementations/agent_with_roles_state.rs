use concordium_protocols::concordium_cis2_security::AgentWithRoles;
use concordium_std::{Address, HasStateApi, Serialize, StateBuilder, StateMap, StateSet};

/// Trait for managing agents in a state.
pub trait IAgentWithRolesState<S: HasStateApi, TAgentRole: Serialize + Copy> {
    /// Returns a reference to the set of agents.
    ///
    /// # Returns
    ///
    /// A reference to the set of agents.
    fn agents(&self) -> &StateMap<Address, StateSet<TAgentRole, S>, S>;

    /// Returns a mutable reference to the set of agents.
    ///
    /// # Returns
    ///
    /// A mutable reference to the set of agents.
    fn agents_mut(&mut self) -> &mut StateMap<Address, StateSet<TAgentRole, S>, S>;

    /// Checks if the given address is an agent.
    ///
    /// # Arguments
    ///
    /// * `address` - An address to check.
    /// * `roles` - A vector of roles to check.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the address is an agent.
    fn is_agent(&self, address: &Address, roles: Vec<TAgentRole>) -> bool {
        self.agents()
            .get(address)
            .is_some_and(|roles_set| roles.iter().all(|r| roles_set.contains(r)))
    }

    /// Adds the given address to the set of agents.
    ///
    /// # Arguments
    ///
    /// * `agent` - Address with roles to add.
    ///
    /// # Returns
    ///
    /// If the agent did not exist, returns `true`. Otherwise, returns `false`.
    fn add_agent(
        &mut self,
        agent: AgentWithRoles<TAgentRole>,
        state_builder: &mut StateBuilder<S>,
    ) -> bool {
        let mut roles_set = state_builder.new_set();
        agent.roles.iter().for_each(|r| {
            roles_set.insert(*r);
        });
        self.agents_mut().insert(agent.address, roles_set).is_none()
    }

    /// Removes the given address from the set of agents.
    ///
    /// # Arguments
    ///
    /// * `agent` - An address to remove.
    ///
    /// # Returns
    ///
    /// Removes a value from the set. Returns whether the value was present in
    /// the set.
    fn remove_agent(&mut self, address: &Address) -> Option<AgentWithRoles<TAgentRole>> {
        self.agents_mut()
            .remove_and_get(address)
            .map(|roles_set| Self::to_struct(&roles_set, *address))
    }

    /// Returns a list of all agents.
    ///
    /// # Returns
    ///
    /// A vector containing all agents.
    fn list_agents(&self) -> Vec<AgentWithRoles<TAgentRole>> {
        self.agents().iter().map(|a| Self::to_struct(&a.1, *a.0)).collect()
    }

    fn to_struct(
        roles_set: &StateSet<TAgentRole, S>,
        address: Address,
    ) -> AgentWithRoles<TAgentRole> {
        let mut roles = Vec::new();
        roles_set.iter().for_each(|r| {
            roles.push(*r);
        });
        AgentWithRoles {
            address,
            roles,
        }
    }
}
