export default function CCDScanModuleLink(props: { moduleRef: string }) {
	const ccdScanWebsite = import.meta.env.VITE_CCDSCAN_MODULE_LINK!.replace(
		"<moduleRef>",
		props.moduleRef,
	);
	return (
		<a href={ccdScanWebsite} target="_blank" rel="noreferrer">
			{props.moduleRef.substring(0, 10)}...
		</a>
	);
}
