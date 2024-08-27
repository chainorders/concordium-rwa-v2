export default function CCDScanAccountLink(props: { account: string }) {
	const ccdScanWebsite = import.meta.env.VITE_CCDSCAN_ACCOUNT_LINK!.replace(
		"<account>",
		props.account,
	);
	return (
		<a href={ccdScanWebsite} target="_blank" rel="noreferrer">
			{props.account.substring(0, 10)}...
		</a>
	);
}
