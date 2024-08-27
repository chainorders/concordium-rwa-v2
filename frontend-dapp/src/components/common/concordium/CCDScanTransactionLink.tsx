import { TransactionHash } from "@concordium/web-sdk";
import { OpenInNew } from "@mui/icons-material";
import { IconButton, Link } from "@mui/material";

export default function CCDScanTransactionLink(props: {
	transactionHash: TransactionHash.Type;
	color?:
		| "inherit"
		| "primary"
		| "secondary"
		| "success"
		| "error"
		| "info"
		| "warning";
	children?: React.ReactNode;
	onClick?: (event: React.MouseEvent<HTMLAnchorElement, MouseEvent>) => void;
}) {
	const ccdScanWebsite = import.meta.env.VITE_CCDSCAN_TRANSACTION_LINK!.replace(
		"<transactionHash>",
		TransactionHash.toHexString(props.transactionHash),
	);
	const children = props.children || <OpenInNew />;

	const txnHashString = TransactionHash.toHexString(props.transactionHash);
	return (
		<IconButton
			href={ccdScanWebsite}
			LinkComponent={Link}
			title={txnHashString}
			target="_blank"
			color={props.color}
			onClick={props.onClick}
		>
			{children}
		</IconButton>
	);
}
