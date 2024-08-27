import { Contract } from "./components/contracts/ContractTypes";
import JSON from "json-bigint";

export interface AppState {
	contracts: Contract[];
}

function setLocalStorage(state: AppState) {
	localStorage.setItem("contracts", JSON.stringify(state.contracts));
}

function getLocalStorage(): AppState {
	return {
		contracts: JSON.parse(localStorage.getItem("contracts") || "[]"),
	};
}

export function initialState(): AppState {
	return getLocalStorage();
}

export enum ActionTypes {
	AddContract = "addContract",
	RemoveContract = "RemoveContract",
}

export type Action = {
	type: ActionTypes.AddContract | ActionTypes.RemoveContract;
	contract: Contract;
};

export function reducer(state: AppState, action: Action): AppState {
	let updatedState: AppState = state;
	switch (action.type) {
		case ActionTypes.AddContract:
			{
				if (
					state.contracts.find(
						(contract) =>
							contract.address.index === action.contract.address.index,
					)
				) {
					return state;
				}

				updatedState = {
					...state,
					contracts: [...state.contracts, action.contract],
				};
			}
			break;
		case ActionTypes.RemoveContract:
			{
				updatedState = {
					...state,
					contracts: state.contracts.filter(
						(contract) => contract.address !== action.contract.address,
					),
				};
			}
			break;
	}

	setLocalStorage(updatedState);
	return updatedState;
}
