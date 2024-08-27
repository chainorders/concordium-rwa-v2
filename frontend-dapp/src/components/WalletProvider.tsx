import {
	EventType,
	WalletApi,
	detectConcordiumProvider,
} from "@concordium/browser-wallet-api-helpers";
import { AccountAddress } from "@concordium/web-sdk";
import { Paper, Typography } from "@mui/material";
import { createContext, useContext, useEffect, useState } from "react";

export type WalletContext = {
	provider?: WalletApi;
	currentAccount?: AccountAddress.Type;
	accounts?: string[];
};

const createWalletContext = async (): Promise<WalletContext> => {
	const provider = await detectConcordiumProvider();
	if (!provider) {
		throw new Error("No Concordium Wallet detected");
	}

	const accounts = await provider.requestAccounts();
	const currentAccount = await provider.getMostRecentlySelectedAccount();
	if (!currentAccount) {
		throw new Error("No account selected");
	}

	return {
		provider,
		currentAccount: AccountAddress.fromBase58(currentAccount),
		accounts,
	};
};
const defaultWalletContext: WalletContext = {};
const WalletContext = createContext<WalletContext>(defaultWalletContext);
export const useWallet = () => {
	return useContext(WalletContext);
};

export default function ConcordiumWalletProvider({
	children,
}: {
	children: React.ReactNode;
}): React.ReactNode {
	const [wallet, setWallet] = useState<WalletContext>(defaultWalletContext);
	const [error, setError] = useState<Error | undefined>();
	const [loading, setLoading] = useState<boolean>(true);

	useEffect(() => {
		setLoading(true);
		createWalletContext()
			.then((context) => {
				setWallet(context);
				setError(undefined);

				context.provider?.addListener(EventType.AccountChanged, (account) => {
					setWallet((wallet) => {
						return {
							...wallet,
							currentAccount: AccountAddress.fromBase58(account),
							account,
						};
					});
				});

				context.provider?.addListener(
					EventType.AccountDisconnected,
					(account) => {
						setWallet((wallet) => {
							return {
								...wallet,
								currentAccount:
									wallet.currentAccount?.address === account
										? undefined
										: wallet.currentAccount,
								accounts: wallet.accounts?.filter((a) => a !== account),
							};
						});
					},
				);
			})
			.catch(setError)
			.finally(() => setLoading(false));
	}, []);

	if (loading) {
		return (
			<Paper>
				<Typography variant="h2">Connecting to Concordium Wallet</Typography>
			</Paper>
		);
	}

	if (error) {
		console.error(error);
		return (
			<Paper>
				<Typography variant="h2">
					Error connecting to Concordium Wallet
				</Typography>
				<Typography variant="body1" color="error">
					{error.message}
				</Typography>
			</Paper>
		);
	}

	if (!wallet.provider) {
		return (
			<Paper>
				<Typography variant="h2">No Concordium Wallet detected</Typography>
			</Paper>
		);
	}

	if (!wallet.currentAccount) {
		return (
			<Paper>
				<Typography variant="h2">No account selected</Typography>
			</Paper>
		);
	}

	return (
		<WalletContext.Provider value={wallet}>{children}</WalletContext.Provider>
	);
}
