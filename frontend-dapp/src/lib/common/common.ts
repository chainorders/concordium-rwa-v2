import {
	BlockItemSummaryInBlock,
	ContractAddress,
	RejectReasonTag,
	TransactionKindString,
	TransactionSummaryType,
} from "@concordium/web-sdk";

export function parseContractAddress(
	outcome: BlockItemSummaryInBlock,
): ContractAddress.Type {
	switch (outcome.summary.type) {
		case TransactionSummaryType.AccountTransaction:
			switch (outcome.summary.transactionType) {
				case TransactionKindString.InitContract:
					return outcome.summary.contractInitialized.address;
				case TransactionKindString.Failed: {
					switch (outcome.summary.rejectReason.tag) {
						case RejectReasonTag.RejectedInit: {
							throw new Error(
								`Rejected init: ${outcome.summary.rejectReason.rejectReason}`,
							);
						}
						default:
							throw new Error(
								`Unknown reject reason ${outcome.summary.rejectReason.tag}`,
							);
					}
				}
				default:
					throw new Error(
						`Unknown account transaction type: ${outcome.summary.transactionType}`,
					);
			}
		default:
			throw new Error(`Unknown transaction type: ${outcome.summary.type}`);
	}
}

export function contractAddToString(
	contractAddress: ContractAddress.Type,
): string {
	return `${contractAddress.index.toString()}/${contractAddress.subindex.toString()}`;
}

export type ParsedError<TErr> = {
	message: string;
	error?: TErr;
};
