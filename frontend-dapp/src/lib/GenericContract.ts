import {
	SchemaSource,
	SendTransactionInitContractPayload,
	SendTransactionUpdateContractPayload,
	SmartContractParameters,
	WalletApi,
} from "@concordium/browser-wallet-api-helpers";
import {
	AccountAddress,
	AccountTransactionType,
	CcdAmount,
	ConcordiumGRPCClient,
	ContractAddress,
	ContractName,
	Energy,
	EntrypointName,
	InvokeContractResult,
	ModuleReference,
	ReceiveName,
	RejectedInit,
	RejectedReceive,
	ReturnValue,
	deserializeTypeValue,
	serializeTypeValue,
} from "@concordium/web-sdk";
import { Buffer } from "buffer/";
import { ParsedError } from "./common/common";

export class InitMethod<TIn> {
	constructor(
		public moduleRef: ModuleReference.Type,
		public contractName: ContractName.Type,
		public paramsSchemaBase64?: string,
		public maxExecutionEnergy: Energy.Type = Energy.create(60000),
	) {}

	async init(
		provider: WalletApi,
		account: AccountAddress.Type,
		params?: TIn,
		amount: CcdAmount.Type = CcdAmount.fromCcd(0),
	) {
		const schema: SchemaSource | undefined = this.paramsSchemaBase64
			? {
					type: "parameter",
					value: this.paramsSchemaBase64,
				}
			: undefined;

		return provider.sendTransaction(
			account,
			AccountTransactionType.InitContract,
			{
				amount,
				moduleRef: this.moduleRef,
				initName: this.contractName,
				maxContractExecutionEnergy: this.maxExecutionEnergy,
			} as SendTransactionInitContractPayload,
			params as SmartContractParameters,
			schema,
		);
	}

	parseError: (value: RejectedInit) => string | undefined = (value) => {
		return `Error Code: ${value.rejectReason}`;
	};
}

export class ReceiveMethod<TIn, TOut = never, TErr = never> {
	constructor(
		public contractName: ContractName.Type,
		public entrypoint: EntrypointName.Type,
		public paramsSchemaBase64?: string,
		public outSchemaBase64?: string,
		public errorSchemaBase64?: string,
		public maxExecutionEnergy: Energy.Type = Energy.create(60000),
	) {}
	async update(
		provider: WalletApi,
		account: AccountAddress.Type,
		address: ContractAddress.Type,
		params?: TIn,
		amount: CcdAmount.Type = CcdAmount.fromCcd(0),
	): Promise<string> {
		const schema: SchemaSource | undefined = this.paramsSchemaBase64
			? {
					type: "parameter",
					value: this.paramsSchemaBase64,
				}
			: undefined;

		return provider.sendTransaction(
			account,
			AccountTransactionType.Update,
			{
				amount,
				contractName: this.contractName,
				address,
				maxContractExecutionEnergy: this.maxExecutionEnergy,
				receiveName: ReceiveName.create(this.contractName, this.entrypoint),
			} as SendTransactionUpdateContractPayload,
			params as SmartContractParameters,
			schema,
		);
	}

	async invoke(
		provider: ConcordiumGRPCClient,
		contract: ContractAddress.Type,
		params?: TIn,
		invoker?: AccountAddress.Type,
		amount: CcdAmount.Type = CcdAmount.fromCcd(0),
	): Promise<InvokeContractResult> {
		const parameter =
			params &&
			serializeTypeValue(
				params,
				Buffer.from(this.paramsSchemaBase64!, "base64"),
			);

		return await provider.invokeContract({
			contract,
			parameter: parameter ? parameter : undefined,
			invoker,
			method: ReceiveName.create(this.contractName, this.entrypoint),
			energy: this.maxExecutionEnergy,
			amount,
		});
	}

	parseError: (value: RejectedReceive) => ParsedError<TErr> | undefined = (
		value,
	) => {
		return {
			message: `Error Code: ${value.rejectReason}`,
			error: undefined,
		};
	};

	parseReturnValue: (value: ReturnValue.Type) => TOut | undefined = (value) => {
		if (!this.outSchemaBase64) {
			return undefined;
		}

		return deserializeTypeValue(
			value.buffer,
			Buffer.from(this.outSchemaBase64!, "base64"),
		) as TOut;
	};
}
