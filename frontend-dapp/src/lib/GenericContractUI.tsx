import {
	TransactionStatusEnum,
	TransactionHash,
	BaseAccountTransactionSummary,
	UpdateContractSummary,
	ContractAddress,
	BlockItemSummaryInBlock,
	RejectReasonTag,
	RejectedReceive,
	TransactionKindString,
	TransactionSummaryType,
	RejectReason,
	RejectedInit,
} from "@concordium/web-sdk";
import { CheckCircle } from "@mui/icons-material";
import {
	Stack,
	Typography,
	CircularProgress,
	Button,
	Alert,
} from "@mui/material";
import { useState } from "react";
import { parseUiToContract, parseContractToUi } from "./genericParser";
import { useNodeClient } from "../components/NodeClientProvider";
import { useWallet } from "../components/WalletProvider";
import CCDScanTransactionLink from "../components/common/concordium/CCDScanTransactionLink";
import { InitMethod, ReceiveMethod } from "./GenericContract";
import { Form } from "@rjsf/mui";
import validator from "@rjsf/validator-ajv8";
import { RJSFSchema, RegistryWidgetsType, UiSchema } from "@rjsf/utils";
import { ParsedError } from "./common/common";

type UpdateInitState = {
	type: "init";
	error: string;
	status: undefined;
	txnHash: undefined;
};

type UpdateSentState = {
	type: "sent";
	status: TransactionStatusEnum.Received;
	txnHash: TransactionHash.Type;
	error: string;
};

type UpdateFinalizedSuccessState = {
	type: "finalized";
	status: TransactionStatusEnum.Finalized;
	txnHash: TransactionHash.Type;
	error?: string;
	response: {
		type: "success";
		summary: BaseAccountTransactionSummary & UpdateContractSummary;
	};
};
type UpdateFinalizedErrorState<E> = {
	type: "finalized";
	status: TransactionStatusEnum.Finalized;
	txnHash: TransactionHash.Type;
	error?: string;
	response: { type: "error"; error?: E; message?: string };
};
type UpdateFinalizedState<E> =
	| UpdateFinalizedSuccessState
	| UpdateFinalizedErrorState<E>;

type ParsedFinalizedUpdateSuccess = {
	tag: "success";
	value: BaseAccountTransactionSummary & UpdateContractSummary;
};
type ParsedFinalizedContractUpdateTxnError = {
	tag: "error";
	value: RejectedReceive;
};

const parseFinalizedUpdate: (
	txnSummary: BlockItemSummaryInBlock,
) => ParsedFinalizedUpdateSuccess | ParsedFinalizedContractUpdateTxnError = (
	txnSummary,
) => {
	switch (txnSummary.summary.type) {
		case TransactionSummaryType.AccountTransaction: {
			switch (txnSummary.summary.transactionType) {
				case TransactionKindString.Update: {
					return { tag: "success", value: txnSummary.summary };
				}
				case TransactionKindString.Failed: {
					switch (txnSummary.summary.rejectReason.tag) {
						case RejectReasonTag.RejectedReceive: {
							return { tag: "error", value: txnSummary.summary.rejectReason };
						}
						default:
							throw new Error("Unknown reject reason");
					}
				}
				default:
					throw new Error("Unknown account transaction type");
			}
			break;
		}
		default:
			throw new Error("Unknown transaction type");
	}
};

export interface GenericUpdateRequestProps<TReq, TError> {
	contract: ContractAddress.Type;
	method: ReceiveMethod<TReq, unknown, TError>;
	uiSchema?: UiSchema;
	uiWidgets?: RegistryWidgetsType;
	requestSchemaBase64?: string;
	requestJsonSchema?: RJSFSchema;
	errorSchemaBase64?: string;
	errorJsonSchema?: RJSFSchema;
}
export function GenericUpdate<TReq, TReqUi, TError, TErrorUi>(
	props: GenericUpdateRequestProps<TReq, TError>,
) {
	const { provider } = useNodeClient();
	const wallet = useWallet();

	const [state, setState] = useState<
		UpdateInitState | UpdateSentState | UpdateFinalizedState<TErrorUi>
	>({
		type: "init",
		error: "",
		status: undefined,
		txnHash: undefined,
	});
	const resetState = () => {
		setState({
			type: "init",
			error: "",
			status: undefined,
			txnHash: undefined,
		});
	};
	const [formData, setFormData] = useState<TReqUi>();
	const onRequest = async (formData: TReqUi) => {
		resetState();
		try {
			const contractRequest = props.requestSchemaBase64
				? parseUiToContract<TReqUi, TReq>(formData, props.requestSchemaBase64)
				: undefined;
			const txnHash = await props.method
				.update(
					wallet.provider!,
					wallet.currentAccount!,
					props.contract,
					contractRequest,
				)
				.then(TransactionHash.fromHexString);
			setState({
				type: "sent",
				status: TransactionStatusEnum.Received,
				txnHash,
				error: "",
			});

			const outcome = await provider.waitForTransactionFinalization(txnHash);
			const result = parseFinalizedUpdate(outcome);
			switch (result.tag) {
				case "success":
					setState({
						type: "finalized",
						status: TransactionStatusEnum.Finalized,
						txnHash,
						response: {
							type: "success",
							summary: result.value,
						},
						error: undefined,
					});
					setFormData(undefined);
					break;
				case "error": {
					setState({
						type: "finalized",
						status: TransactionStatusEnum.Finalized,
						txnHash,
						response: {
							type: "error",
							message: `Reject Reason: ${result.value.rejectReason}`,
						},
						error: undefined,
					});
					break;
				}
			}
		} catch (error) {
			setState({
				type: "init",
				error: error instanceof Error ? error.message : JSON.stringify(error),
				status: undefined,
				txnHash: undefined,
			});
		}
	};

	return (
		<Stack spacing={2}>
			{
				{
					init: (
						<Form
							formData={formData}
							onChange={(e) => setFormData(e.formData)}
							schema={props.requestJsonSchema || {}}
							validator={validator}
							onSubmit={(v) => onRequest(v.formData)}
							uiSchema={props.uiSchema}
							widgets={props.uiWidgets}
						/>
					),
					sent: (
						<>
							<Typography pr={1}>Transaction {state.status!}</Typography>
							<CircularProgress size={10} />
						</>
					),
					finalized: (
						<>
							<Typography>
								Transaction Hash:{" "}
								<CCDScanTransactionLink transactionHash={state.txnHash!} />
							</Typography>
							{
								{
									success: (
										<>
											<Button
												variant="contained"
												onClick={resetState}
												color="success"
												endIcon={<CheckCircle />}
											>
												Transaction {state.status!}
											</Button>
										</>
									),
									error: (
										<>
											{(state as UpdateFinalizedErrorState<TErrorUi>).response
												?.message && (
												<Typography color="error">
													{
														(state as UpdateFinalizedErrorState<TErrorUi>)
															.response?.message
													}
												</Typography>
											)}
											<Button
												variant="contained"
												onClick={resetState}
												color="error"
												endIcon={<CheckCircle />}
											>
												Transaction{state.status!}
											</Button>
										</>
									),
								}[(state as UpdateFinalizedState<TErrorUi>).response?.type]
							}
						</>
					),
				}[state.type]
			}
			{state.error && <Typography color="error">{state.error}</Typography>}
		</Stack>
	);
}

// Invoke
type InvokeInitState = {
	type: "init";
	error: string;
};

type InvokeSentState = {
	type: "sent";
	error: string;
};

type InvokeResponseState<T, E> =
	| InvokeResponseSuccessState<T>
	| InvokeResponseErrorState<E>;

type InvokeResponseSuccessState<T> = {
	type: "response";
	value: {
		type: "success";
		value?: T;
	};
	error: string;
};

type InvokeResponseErrorState<T> = {
	type: "response";
	value: {
		type: "error";
		value?: T;
		message?: string;
	};
	error: string;
};

const parseInvokeError = (error: RejectReason) => {
	switch (error.tag) {
		case RejectReasonTag.RejectedReceive:
			return error;
		default:
			throw new Error("Unknown reject reason");
	}
};

export interface GenericInvokeRequestProps<TReq, TRes, TError> {
	contract: ContractAddress.Type;
	method: ReceiveMethod<TReq, TRes, TError>;
	requestJsonSchema?: RJSFSchema;
	requestSchemaBase64?: string;
	responseJsonSchema?: RJSFSchema;
	responseSchemaBase64?: string;
	errorJsonSchema?: RJSFSchema;
	errorSchemaBase64?: string;
	uiSchema?: UiSchema;
	uiWidgets?: RegistryWidgetsType;
}
export function GenericInvoke<TReq, TReqUi, TRes, TResUi, TError, TErrorUi>(
	props: GenericInvokeRequestProps<TReq, TRes, TError>,
) {
	const { provider } = useNodeClient();
	const wallet = useWallet();

	const [state, setState] = useState<
		InvokeInitState | InvokeSentState | InvokeResponseState<TResUi, TErrorUi>
	>({
		type: "init",
		error: "",
	});
	const resetState = () => {
		setState({
			type: "init",
			error: "",
		});
	};
	const [formData, setFormData] = useState<TReqUi>();
	const onRequest = async (formData: TReqUi) => {
		resetState();
		try {
			const contractRequest = props.requestSchemaBase64
				? parseUiToContract<TReqUi, TReq>(formData, props.requestSchemaBase64)
				: undefined;
			const result = await props.method.invoke(
				provider!,
				props.contract,
				contractRequest,
				wallet.currentAccount!,
			);
			switch (result.tag) {
				case "success": {
					try {
						const contractResult = result.returnValue
							? props.method.parseReturnValue(result.returnValue)
							: undefined;
						// console.log(
						// 	"contract result",
						// 	props.method.entrypoint.value,
						// 	contractResult,
						// );
						// console.log("response schema", props.responseSchemaBase64);
						const uiResult: TResUi | undefined =
							contractResult != null &&
							contractResult != undefined &&
							props.responseSchemaBase64
								? parseContractToUi(contractResult, props.responseSchemaBase64)
								: undefined;
						// console.log("ui result", props.method.entrypoint.value, uiResult);
						setState({
							type: "response",
							value: {
								type: "success",
								value: uiResult,
							},
							error: "",
						});
					} catch (error) {
						setState({
							type: "response",
							value: {
								type: "success",
							},
							error:
								error instanceof Error ? error.message : JSON.stringify(error),
						});
					}
					break;
				}
				case "failure": {
					try {
						const contractError: ParsedError<TError> | undefined =
							props.method.parseError?.(parseInvokeError(result.reason));
						setState({
							type: "response",
							value: {
								type: "error",
								message: contractError?.message,
							},
							error: "",
						});
					} catch (error) {
						setState({
							type: "response",
							value: {
								type: "error",
							},
							error:
								error instanceof Error ? error.message : JSON.stringify(error),
						});
					}
					break;
				}
			}
		} catch (error) {
			setState({
				type: "init",
				error: error instanceof Error ? error.message : JSON.stringify(error),
			});
		}
	};

	return (
		<Stack spacing={2}>
			{
				{
					init: (
						<Form
							formData={formData}
							onChange={(e) => setFormData(e.formData)}
							schema={props.requestJsonSchema || {}}
							validator={validator}
							onSubmit={(v) => onRequest(v.formData)}
							uiSchema={props.uiSchema}
							widgets={props.uiWidgets}
						/>
					),
					sent: (
						<>
							<Typography pr={1}>Invoke Request Sent</Typography>
							<CircularProgress size={10} />
						</>
					),
					response: (
						<>
							{
								{
									success: (
										<>
											<Alert severity="success">Invoke Request Success</Alert>
											<Form
												schema={props.responseJsonSchema || {}}
												validator={validator}
												formData={
													(state as InvokeResponseSuccessState<TResUi>).value
														?.value
												}
												readonly
												onSubmit={resetState}
											/>
										</>
									),
									error: (
										<>
											<Alert severity="error">Invoke Request Error</Alert>
											{(state as InvokeResponseErrorState<TErrorUi>).value
												?.value && (
												<Form
													schema={props.errorJsonSchema || {}}
													validator={validator}
													formData={
														(state as InvokeResponseErrorState<TErrorUi>).value
															?.value
													}
													readonly
													onSubmit={resetState}
												/>
											)}
											{(state as InvokeResponseErrorState<TErrorUi>).value
												?.message && (
												<Typography color="error">
													{
														(state as InvokeResponseErrorState<TErrorUi>).value
															?.message
													}
												</Typography>
											)}
											<Button
												variant="contained"
												onClick={resetState}
												color="error"
												endIcon={<CheckCircle />}
											>
												Ok
											</Button>
										</>
									),
								}[(state as InvokeResponseState<TResUi, TErrorUi>).value?.type]
							}
						</>
					),
				}[state.type]
			}
			{state.error && <Typography color="error">{state.error}</Typography>}
		</Stack>
	);
}

type InitInitState = UpdateInitState;
type InitSentState = UpdateSentState;
type InitFinalizedSuccessState = {
	type: "finalized";
	status: TransactionStatusEnum.Finalized;
	txnHash: TransactionHash.Type;
	error?: string;
	response: { type: "success"; address: ContractAddress.Type };
};
type InitFinalizedErrorState = UpdateFinalizedErrorState<never>;
type InitFinalizedState = InitFinalizedSuccessState | InitFinalizedErrorState;

type ParsedFinalizedInitSuccess = {
	tag: "success";
	value: ContractAddress.Type;
};
type ParsedFinalizedContractInitTxnError = {
	tag: "error";
	value: RejectedInit;
};

const parseFinalizedInit: (
	txnSummary: BlockItemSummaryInBlock,
) => ParsedFinalizedInitSuccess | ParsedFinalizedContractInitTxnError = (
	txnSummary,
) => {
	switch (txnSummary.summary.type) {
		case TransactionSummaryType.AccountTransaction: {
			switch (txnSummary.summary.transactionType) {
				case TransactionKindString.InitContract: {
					return {
						tag: "success",
						value: txnSummary.summary.contractInitialized.address,
					};
				}
				case TransactionKindString.Failed: {
					switch (txnSummary.summary.rejectReason.tag) {
						case RejectReasonTag.RejectedInit: {
							return { tag: "error", value: txnSummary.summary.rejectReason };
						}
						default:
							throw new Error(
								`Unknown reject reason ${txnSummary.summary.rejectReason.tag}`,
							);
					}
				}
				default:
					throw new Error(
						`"Unknown account transaction type: ${txnSummary.summary.transactionType}`,
					);
			}
			break;
		}
		default:
			throw new Error(`Unknown transaction type: ${txnSummary.summary.type}`);
	}
};

export interface GenericInitRequestProps<TReq> {
	onContractInitialized: (contract: ContractAddress.Type) => void;
	method: InitMethod<TReq>;
	requestSchemaBase64?: string;
	requestJsonSchema?: RJSFSchema;
	uiSchema?: UiSchema;
	uiWidgets?: RegistryWidgetsType;
}
export function GenericInit<TReq, TReqUi>(
	props: GenericInitRequestProps<TReq>,
) {
	const { provider } = useNodeClient();
	const wallet = useWallet();

	const [state, setState] = useState<
		InitInitState | InitSentState | InitFinalizedState
	>({
		type: "init",
		error: "",
		status: undefined,
		txnHash: undefined,
	});
	const resetState = () => {
		setState({
			type: "init",
			error: "",
			status: undefined,
			txnHash: undefined,
		});
	};
	const [formData, setFormData] = useState<TReqUi>();
	const onRequest = async (formData: TReqUi) => {
		resetState();
		try {
			const contractRequest = props.requestSchemaBase64
				? parseUiToContract<TReqUi, TReq>(formData, props.requestSchemaBase64)
				: undefined;
			const txnHash = await props.method
				.init(wallet.provider!, wallet.currentAccount!, contractRequest)
				.then(TransactionHash.fromHexString);
			setState({
				type: "sent",
				status: TransactionStatusEnum.Received,
				txnHash,
				error: "",
			});

			const outcome = await provider.waitForTransactionFinalization(txnHash);
			const result = parseFinalizedInit(outcome);
			switch (result.tag) {
				case "success":
					setState({
						type: "finalized",
						status: TransactionStatusEnum.Finalized,
						txnHash,
						response: {
							type: "success",
							address: result.value,
						},
						error: undefined,
					});
					setFormData(undefined);
					props.onContractInitialized(result.value);
					break;
				case "error": {
					try {
						const contractError = props.method.parseError?.(result.value);
						setState({
							type: "finalized",
							status: TransactionStatusEnum.Finalized,
							txnHash,
							response: {
								type: "error",
								message: contractError,
							},
							error: undefined,
						});
					} catch (error) {
						setState({
							type: "finalized",
							status: TransactionStatusEnum.Finalized,
							txnHash,
							response: {
								type: "error",
							},
							error:
								error instanceof Error ? error.message : JSON.stringify(error),
						});
					}
					break;
				}
			}
		} catch (error) {
			setState({
				type: "init",
				error: error instanceof Error ? error.message : JSON.stringify(error),
				status: undefined,
				txnHash: undefined,
			});
		}
	};

	return (
		<Stack spacing={2}>
			{
				{
					init: (
						<Form
							formData={formData}
							onChange={(e) => setFormData(e.formData)}
							schema={props.requestJsonSchema || {}}
							validator={validator}
							onSubmit={(v) => onRequest(v.formData)}
							uiSchema={props.uiSchema}
							widgets={props.uiWidgets}
						/>
					),
					sent: (
						<>
							<Typography pr={1}>Transaction {state.status!}</Typography>
							<CircularProgress size={10} />
						</>
					),
					finalized: (
						<>
							<Typography>
								Transaction Hash:{" "}
								<CCDScanTransactionLink transactionHash={state.txnHash!} />
							</Typography>
							{
								{
									success: (
										<>
											<Button
												variant="contained"
												onClick={resetState}
												endIcon={<CheckCircle />}
											>
												Transaction {state.status!}
											</Button>
										</>
									),
									error: (
										<Alert severity="error">
											{(state as InitFinalizedErrorState).response?.message ||
												"Unknown error"}
										</Alert>
									),
								}[(state as InitFinalizedState).response?.type]
							}
						</>
					),
				}[state.type]
			}
			{state.error && <Typography color="error">{state.error}</Typography>}
		</Stack>
	);
}
