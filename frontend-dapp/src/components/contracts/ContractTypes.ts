import { ContractAddress } from "@concordium/web-sdk";

export interface Contract {
	name: string;
	address: ContractAddress.Type;
	type: string;
}
