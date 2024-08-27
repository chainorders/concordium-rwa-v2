import { ConcordiumGRPCWebClient } from "@concordium/web-sdk";
import { createContext, useContext } from "react";
import { default as defaultNodeClient } from "../lib/ConcordiumNodeClient";

export type NodeClientContext = {
	provider: ConcordiumGRPCWebClient;
};

const NodeClientContext = createContext<NodeClientContext>({
	provider: defaultNodeClient,
});

export const useNodeClient = () => {
	return useContext(NodeClientContext);
};

export default function ConcordiumNodeClientProvider({
	children,
}: {
	children: React.ReactNode;
}): React.ReactNode {
	return (
		<NodeClientContext.Provider
			value={{
				provider: defaultNodeClient,
			}}
		>
			{children}
		</NodeClientContext.Provider>
	);
}
