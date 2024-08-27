use concordium_std::{
    ContractAddress, DeserialWithState, Serial, StateApi, StateBuilder, StateSet,
};

use super::types::AttributeValue;

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S=StateApi> {
    allowed_nationalities: StateSet<AttributeValue, S>,
    identity_registry:     ContractAddress,
}

impl State<StateApi> {
    pub fn new(
        identity_registry: ContractAddress,
        nationalities: Vec<AttributeValue>,
        state_builder: &mut StateBuilder,
    ) -> Self {
        let mut state = Self {
            allowed_nationalities: state_builder.new_set(),
            identity_registry,
        };

        for nationality in nationalities {
            state.allowed_nationalities.insert(nationality);
        }

        state
    }

    pub fn identity_registry(&self) -> ContractAddress { self.identity_registry }

    pub fn is_allowed(&self, value: AttributeValue) -> bool {
        self.allowed_nationalities.contains(&value)
    }
}
