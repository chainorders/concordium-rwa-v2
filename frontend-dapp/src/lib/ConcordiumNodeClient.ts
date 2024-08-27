import * as concordiumWebSdk from "@concordium/web-sdk";
const concordiumNodeClient = new concordiumWebSdk.ConcordiumGRPCWebClient(
	import.meta.env.VITE_CONCORDIUM_NODE_URL!,
	Number(import.meta.env.VITE_CONCORDIUM_NODE_PORT!),
);

export default concordiumNodeClient;
