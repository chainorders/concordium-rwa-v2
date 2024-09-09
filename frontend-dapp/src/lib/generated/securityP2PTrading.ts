// This file was generated by the build script. Do not edit it manually.
/* eslint-disable no-mixed-spaces-and-tabs */
/* eslint-disable @typescript-eslint/no-unused-vars */
/* tslint:disable:no-unused-variable */
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck
import {
	ContractEvent,
	ContractName,
	EntrypointName,
	ModuleReference,
} from "@concordium/web-sdk";
import { InitMethod, ReceiveMethod } from "../GenericContract";
export const CONTRACT_NAME = "security_p2p_trading";
export type initRequest = {
	token: { contract: { index: number; subindex: number }; id: string };
	currency: { contract: { index: number; subindex: number }; id: string };
};
export const initRequestSchemaBase64 =
	"FAACAAAABQAAAHRva2VuFAACAAAACAAAAGNvbnRyYWN0DAIAAABpZB0ACAAAAGN1cnJlbmN5FAACAAAACAAAAGNvbnRyYWN0DAIAAABpZB0A";
export const initErrorSchemaBase64 =
	"FQkAAAAKAAAAUGFyc2VFcnJvcgIMAAAAVW5hdXRob3JpemVkAg0AAABDaXMyQ2FsbEVycm9yAgwAAABJbnZhbGlkVG9rZW4CEgAAAFNlbGxQb3NpdGlvbkV4aXN0cwITAAAAU2VsbFBvc2l0aW9uTWlzc2luZwIRAAAASW52YWxpZENvbnZlcnNpb24CDQAAAEludmFsaWRBbW91bnQCCAAAAExvZ0Vycm9yAg==";
export type CancelSellError =
	| { ParseError: Record<string, never> }
	| { Unauthorized: Record<string, never> }
	| { Cis2CallError: Record<string, never> }
	| { InvalidToken: Record<string, never> }
	| { SellPositionExists: Record<string, never> }
	| { SellPositionMissing: Record<string, never> }
	| { InvalidConversion: Record<string, never> }
	| { InvalidAmount: Record<string, never> }
	| { LogError: Record<string, never> };
export const cancelSellErrorSchemaBase64 =
	"FQkAAAAKAAAAUGFyc2VFcnJvcgIMAAAAVW5hdXRob3JpemVkAg0AAABDaXMyQ2FsbEVycm9yAgwAAABJbnZhbGlkVG9rZW4CEgAAAFNlbGxQb3NpdGlvbkV4aXN0cwITAAAAU2VsbFBvc2l0aW9uTWlzc2luZwIRAAAASW52YWxpZENvbnZlcnNpb24CDQAAAEludmFsaWRBbW91bnQCCAAAAExvZ0Vycm9yAg==";
export type ExchangeError =
	| { ParseError: Record<string, never> }
	| { Unauthorized: Record<string, never> }
	| { Cis2CallError: Record<string, never> }
	| { InvalidToken: Record<string, never> }
	| { SellPositionExists: Record<string, never> }
	| { SellPositionMissing: Record<string, never> }
	| { InvalidConversion: Record<string, never> }
	| { InvalidAmount: Record<string, never> }
	| { LogError: Record<string, never> };
export const exchangeErrorSchemaBase64 =
	"FQkAAAAKAAAAUGFyc2VFcnJvcgIMAAAAVW5hdXRob3JpemVkAg0AAABDaXMyQ2FsbEVycm9yAgwAAABJbnZhbGlkVG9rZW4CEgAAAFNlbGxQb3NpdGlvbkV4aXN0cwITAAAAU2VsbFBvc2l0aW9uTWlzc2luZwIRAAAASW52YWxpZENvbnZlcnNpb24CDQAAAEludmFsaWRBbW91bnQCCAAAAExvZ0Vycm9yAg==";
export type ExchangeRequest = {
	token_id: string;
	amount: string;
	from:
		| { Account: [string] }
		| { Contract: [{ index: number; subindex: number }] };
	data: { from: string; rate: { numerator: bigint; denominator: bigint } };
};
export const exchangeRequestSchemaBase64 =
	"FAAEAAAACAAAAHRva2VuX2lkHQAGAAAAYW1vdW50GyUAAAAEAAAAZnJvbRUCAAAABwAAAEFjY291bnQBAQAAAAsIAAAAQ29udHJhY3QBAQAAAAwEAAAAZGF0YRQAAgAAAAQAAABmcm9tCwQAAAByYXRlFAACAAAACQAAAG51bWVyYXRvcgULAAAAZGVub21pbmF0b3IF";
export type ForceCancelSellError =
	| { ParseError: Record<string, never> }
	| { Unauthorized: Record<string, never> }
	| { Cis2CallError: Record<string, never> }
	| { InvalidToken: Record<string, never> }
	| { SellPositionExists: Record<string, never> }
	| { SellPositionMissing: Record<string, never> }
	| { InvalidConversion: Record<string, never> }
	| { InvalidAmount: Record<string, never> }
	| { LogError: Record<string, never> };
export const forceCancelSellErrorSchemaBase64 =
	"FQkAAAAKAAAAUGFyc2VFcnJvcgIMAAAAVW5hdXRob3JpemVkAg0AAABDaXMyQ2FsbEVycm9yAgwAAABJbnZhbGlkVG9rZW4CEgAAAFNlbGxQb3NpdGlvbkV4aXN0cwITAAAAU2VsbFBvc2l0aW9uTWlzc2luZwIRAAAASW52YWxpZENvbnZlcnNpb24CDQAAAEludmFsaWRBbW91bnQCCAAAAExvZ0Vycm9yAg==";
export type ForceCancelSellRequest = { from: string; to: string };
export const forceCancelSellRequestSchemaBase64 =
	"FAACAAAABAAAAGZyb20LAgAAAHRvCw==";
export type GetDepositRequest = { from: string };
export const getDepositRequestSchemaBase64 = "FAABAAAABAAAAGZyb20L";
export type GetDepositResponse = {
	amount: string;
	rate: { numerator: bigint; denominator: bigint };
};
export const getDepositResponseSchemaBase64 =
	"FAACAAAABgAAAGFtb3VudBslAAAABAAAAHJhdGUUAAIAAAAJAAAAbnVtZXJhdG9yBQsAAABkZW5vbWluYXRvcgU=";
export type SellError =
	| { ParseError: Record<string, never> }
	| { Unauthorized: Record<string, never> }
	| { Cis2CallError: Record<string, never> }
	| { InvalidToken: Record<string, never> }
	| { SellPositionExists: Record<string, never> }
	| { SellPositionMissing: Record<string, never> }
	| { InvalidConversion: Record<string, never> }
	| { InvalidAmount: Record<string, never> }
	| { LogError: Record<string, never> };
export const sellErrorSchemaBase64 =
	"FQkAAAAKAAAAUGFyc2VFcnJvcgIMAAAAVW5hdXRob3JpemVkAg0AAABDaXMyQ2FsbEVycm9yAgwAAABJbnZhbGlkVG9rZW4CEgAAAFNlbGxQb3NpdGlvbkV4aXN0cwITAAAAU2VsbFBvc2l0aW9uTWlzc2luZwIRAAAASW52YWxpZENvbnZlcnNpb24CDQAAAEludmFsaWRBbW91bnQCCAAAAExvZ0Vycm9yAg==";
export type SellRequest = {
	token_id: string;
	amount: string;
	from:
		| { Account: [string] }
		| { Contract: [{ index: number; subindex: number }] };
	data: { numerator: bigint; denominator: bigint };
};
export const sellRequestSchemaBase64 =
	"FAAEAAAACAAAAHRva2VuX2lkHQAGAAAAYW1vdW50GyUAAAAEAAAAZnJvbRUCAAAABwAAAEFjY291bnQBAQAAAAsIAAAAQ29udHJhY3QBAQAAAAwEAAAAZGF0YRQAAgAAAAkAAABudW1lcmF0b3IFCwAAAGRlbm9taW5hdG9yBQ==";
export type TransferExchangeError =
	| { ParseError: Record<string, never> }
	| { Unauthorized: Record<string, never> }
	| { Cis2CallError: Record<string, never> }
	| { InvalidToken: Record<string, never> }
	| { SellPositionExists: Record<string, never> }
	| { SellPositionMissing: Record<string, never> }
	| { InvalidConversion: Record<string, never> }
	| { InvalidAmount: Record<string, never> }
	| { LogError: Record<string, never> };
export const transferExchangeErrorSchemaBase64 =
	"FQkAAAAKAAAAUGFyc2VFcnJvcgIMAAAAVW5hdXRob3JpemVkAg0AAABDaXMyQ2FsbEVycm9yAgwAAABJbnZhbGlkVG9rZW4CEgAAAFNlbGxQb3NpdGlvbkV4aXN0cwITAAAAU2VsbFBvc2l0aW9uTWlzc2luZwIRAAAASW52YWxpZENvbnZlcnNpb24CDQAAAEludmFsaWRBbW91bnQCCAAAAExvZ0Vycm9yAg==";
export type TransferExchangeRequest = {
	pay: string;
	get: { from: string; rate: { numerator: bigint; denominator: bigint } };
};
export const transferExchangeRequestSchemaBase64 =
	"FAACAAAAAwAAAHBheRslAAAAAwAAAGdldBQAAgAAAAQAAABmcm9tCwQAAAByYXRlFAACAAAACQAAAG51bWVyYXRvcgULAAAAZGVub21pbmF0b3IF";
export type TransferSellError =
	| { ParseError: Record<string, never> }
	| { Unauthorized: Record<string, never> }
	| { Cis2CallError: Record<string, never> }
	| { InvalidToken: Record<string, never> }
	| { SellPositionExists: Record<string, never> }
	| { SellPositionMissing: Record<string, never> }
	| { InvalidConversion: Record<string, never> }
	| { InvalidAmount: Record<string, never> }
	| { LogError: Record<string, never> };
export const transferSellErrorSchemaBase64 =
	"FQkAAAAKAAAAUGFyc2VFcnJvcgIMAAAAVW5hdXRob3JpemVkAg0AAABDaXMyQ2FsbEVycm9yAgwAAABJbnZhbGlkVG9rZW4CEgAAAFNlbGxQb3NpdGlvbkV4aXN0cwITAAAAU2VsbFBvc2l0aW9uTWlzc2luZwIRAAAASW52YWxpZENvbnZlcnNpb24CDQAAAEludmFsaWRBbW91bnQCCAAAAExvZ0Vycm9yAg==";
export type TransferSellRequest = {
	amount: string;
	rate: { numerator: bigint; denominator: bigint };
};
export const transferSellRequestSchemaBase64 =
	"FAACAAAABgAAAGFtb3VudBslAAAABAAAAHJhdGUUAAIAAAAJAAAAbnVtZXJhdG9yBQsAAABkZW5vbWluYXRvcgU=";
export type event =
	| {
			Initialized: [
				{
					token: { contract: { index: number; subindex: number }; id: string };
					currency: {
						contract: { index: number; subindex: number };
						id: string;
					};
				},
			];
	  }
	| {
			Sell: [
				{
					from: string;
					amount: string;
					rate: { numerator: bigint; denominator: bigint };
				},
			];
	  }
	| { SellCancelled: [{ from: string; amount: string }] }
	| {
			Exchange: [
				{
					payer: string;
					pay_amount: string;
					sell_amount: string;
					seller: string;
				},
			];
	  };
export const eventSchemaBase64 =
	"FQQAAAALAAAASW5pdGlhbGl6ZWQBAQAAABQAAgAAAAUAAAB0b2tlbhQAAgAAAAgAAABjb250cmFjdAwCAAAAaWQdAAgAAABjdXJyZW5jeRQAAgAAAAgAAABjb250cmFjdAwCAAAAaWQdAAQAAABTZWxsAQEAAAAUAAMAAAAEAAAAZnJvbQsGAAAAYW1vdW50GyUAAAAEAAAAcmF0ZRQAAgAAAAkAAABudW1lcmF0b3IFCwAAAGRlbm9taW5hdG9yBQ0AAABTZWxsQ2FuY2VsbGVkAQEAAAAUAAIAAAAEAAAAZnJvbQsGAAAAYW1vdW50GyUAAAAIAAAARXhjaGFuZ2UBAQAAABQABAAAAAUAAABwYXllcgsKAAAAcGF5X2Ftb3VudBslAAAACwAAAHNlbGxfYW1vdW50GyUAAAAGAAAAc2VsbGVyCw==";
export const ENTRYPOINTS: Record<string, EntrypointName.Type> = {
	cancelSell: EntrypointName.fromString("cancelSell"),
	exchange: EntrypointName.fromString("exchange"),
	forceCancelSell: EntrypointName.fromString("forceCancelSell"),
	getDeposit: EntrypointName.fromString("getDeposit"),
	sell: EntrypointName.fromString("sell"),
	transferExchange: EntrypointName.fromString("transferExchange"),
	transferSell: EntrypointName.fromString("transferSell"),
};
export const ENTRYPOINT_DISPLAY_NAMES: Record<string, string> = {
	cancelSell: "Cancel Sell",
	exchange: "Exchange",
	forceCancelSell: "Force Cancel Sell",
	getDeposit: "Get Deposit",
	sell: "Sell",
	transferExchange: "Transfer Exchange",
	transferSell: "Transfer Sell",
};
export const securityP2PTrading = {
	init: new InitMethod<initRequest>(
		ModuleReference.fromHexString(
			"880eb7f7268e366d8683d9cc70a8a0d8d673e88c9d9edce7af37b90f184546a1",
		),
		ContractName.fromString("security_p2p_trading"),
		initRequestSchemaBase64,
	),
	cancelSell: new ReceiveMethod<never, never, CancelSellError>(
		ContractName.fromString("security_p2p_trading"),
		EntrypointName.fromString("cancelSell"),
		undefined,
		undefined,
		cancelSellErrorSchemaBase64,
	),
	exchange: new ReceiveMethod<ExchangeRequest, never, ExchangeError>(
		ContractName.fromString("security_p2p_trading"),
		EntrypointName.fromString("exchange"),
		exchangeRequestSchemaBase64,
		undefined,
		exchangeErrorSchemaBase64,
	),
	forceCancelSell: new ReceiveMethod<
		ForceCancelSellRequest,
		never,
		ForceCancelSellError
	>(
		ContractName.fromString("security_p2p_trading"),
		EntrypointName.fromString("forceCancelSell"),
		forceCancelSellRequestSchemaBase64,
		undefined,
		forceCancelSellErrorSchemaBase64,
	),
	getDeposit: new ReceiveMethod<GetDepositRequest, GetDepositResponse>(
		ContractName.fromString("security_p2p_trading"),
		EntrypointName.fromString("getDeposit"),
		getDepositRequestSchemaBase64,
		getDepositResponseSchemaBase64,
	),
	sell: new ReceiveMethod<SellRequest, never, SellError>(
		ContractName.fromString("security_p2p_trading"),
		EntrypointName.fromString("sell"),
		sellRequestSchemaBase64,
		undefined,
		sellErrorSchemaBase64,
	),
	transferExchange: new ReceiveMethod<
		TransferExchangeRequest,
		never,
		TransferExchangeError
	>(
		ContractName.fromString("security_p2p_trading"),
		EntrypointName.fromString("transferExchange"),
		transferExchangeRequestSchemaBase64,
		undefined,
		transferExchangeErrorSchemaBase64,
	),
	transferSell: new ReceiveMethod<
		TransferSellRequest,
		never,
		TransferSellError
	>(
		ContractName.fromString("security_p2p_trading"),
		EntrypointName.fromString("transferSell"),
		transferSellRequestSchemaBase64,
		undefined,
		transferSellErrorSchemaBase64,
	),
	deserializeEvent: (event: ContractEvent.Type): event => {
		return ContractEvent.parseWithSchemaTypeBase64(
			event,
			eventSchemaBase64,
		) as event;
	},
};
export default securityP2PTrading;
