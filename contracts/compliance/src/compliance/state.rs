use concordium_rwa_utils::state_implementations::agents_state::IsAgentsState;
use concordium_std::*;

use super::types::Module;

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S=StateApi> {
    pub modules: StateSet<Module, S>,
    pub agents:  StateSet<Address, S>,
}

impl State {
    pub fn new(modules: Vec<Module>, state_builder: &mut StateBuilder) -> Self {
        let mut res = Self {
            modules: state_builder.new_set(),
            agents:  state_builder.new_set(),
        };

        for module in modules {
            res.modules.insert(module);
        }

        res
    }

    pub fn add_module(&mut self, module: Module) -> bool { self.modules.insert(module) }

    pub fn remove_module(&mut self, module: &Module) -> bool { self.modules.remove(module) }

    pub fn modules(&self) -> Vec<Module> { self.modules.iter().map(|m| m.to_owned()).collect() }
}

impl IsAgentsState<StateApi> for State {
    fn agents(&self) -> &StateSet<Address, StateApi> { &self.agents }

    fn agents_mut(&mut self) -> &mut StateSet<Address, StateApi> { &mut self.agents }
}
