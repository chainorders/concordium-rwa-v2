import { Link } from "@mui/material";

export default function CCDScanContractLink(props: {
	index: string;
	subIndex: string;
	text?: string;
}) {
	const ccdScanWebsite = import.meta.env
		.VITE_CCDSCAN_CONTRACT_LINK!.replace("<index>", props.index)
		.replace("<subindex>", props.subIndex);
	const altText = props.index + "/" + props.subIndex;

	return (
		<Link
			href={ccdScanWebsite}
			target="_blank"
			rel="noreferrer"
			title={altText}
		>
			{props.text ? props.text : altText}{" "}
		</Link>
	);
}
