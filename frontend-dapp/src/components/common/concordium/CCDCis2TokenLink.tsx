import { ContractAddress } from "@concordium/web-sdk";
import { Link } from "@mui/material";

export default function CCDCis2TokenLink(props: {
	children?: React.ReactNode;
	tokenId: string;
	contract: ContractAddress.Type;
}) {
	const { tokenId, contract, children } = props;
	const ccdScanWebsite = import.meta.env
		.VITE_CCDSCAN_CIS2_TOKEN_LINK!.replace("<tokenId>", props.tokenId)
		.replace("<index>", contract.index.toString())
		.replace("<subindex>", contract.subindex.toString());
	const text = `${tokenId} (${contract.index.toString()}/${contract.subindex.toString()})`;
	return (
		<Link href={ccdScanWebsite} target="_blank" rel="noreferrer" sx={{ p: 1 }}>
			{children ? children : text}
		</Link>
	);
}
