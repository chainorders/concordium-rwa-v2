import {
	AppBar,
	IconButton,
	ListItemIcon,
	Menu,
	MenuItem,
	Paper,
	Toolbar,
	Typography,
} from "@mui/material";
import { Box, useTheme } from "@mui/material";

import { Route, Routes, useNavigate } from "react-router-dom";
import { Contract } from "./ContractTypes";
import { useEffect, useReducer, useState } from "react";
import { ActionTypes, initialState, reducer } from "../../AppState";
import ContractsList from "./ContractsList";
import ConcordiumContract from "./ConcordiumContract";
import ContractLayout from "./ContractLayout";
import ErrorDisplay from "../common/ErrorDisplay";
import { uiCustomizations } from "../../config/theme";
import {
	EventType,
	WalletApi,
	detectConcordiumProvider,
} from "@concordium/browser-wallet-api-helpers";
import {
	AccountAddress,
	ContractAddress,
	EntrypointName,
} from "@concordium/web-sdk";
import { Login, Logout, Error, ArrowDownward } from "@mui/icons-material";
import InfoDisplay from "../common/InfoDisplay";
import { RegistryWidgetsType, UiSchema } from "@rjsf/utils";
import logo from "../../assets/logo.svg";

const ContractsAppBar = (props: {
	onLogin: (account: AccountAddress.Type, wallet: WalletApi) => void;
	onLogout: () => void;
}) => {
	const navigate = useNavigate();
	const [error, setError] = useState("");
	const [account, setAccount] = useState<AccountAddress.Type>();
	const isLoggedIn = account !== undefined;

	const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);

	const handleMenu = (event: React.MouseEvent<HTMLElement>) => {
		setAnchorEl(event.currentTarget);
	};

	const handleClose = () => {
		setAnchorEl(null);
	};

	const login = async () => {
		const provider = await detectConcordiumProvider();
		if (!provider) {
			setError("No Concordium Wallet detected");
			console.error("No Concordium Wallet detected");
			return;
		}
		provider.addListener(EventType.AccountChanged, (newAccount) => {
			props.onLogout();
			setAccount(AccountAddress.fromBase58(newAccount));
			props.onLogin(AccountAddress.fromBase58(newAccount), provider);
		});
		provider.addListener(EventType.AccountDisconnected, () => {
			setAccount(undefined);
			props.onLogout();
		});
		//login process

		const currentAccount = await provider.getMostRecentlySelectedAccount();
		if (!currentAccount) {
			const accounts = await provider.requestAccounts();
			if (accounts.length === 0) {
				setError("No account selected");
				console.error("No account selected");
				return;
			}
			const account = AccountAddress.fromBase58(accounts[0]);
			setAccount(account);
			props.onLogin(account, provider);
		} else {
			const account = AccountAddress.fromBase58(currentAccount);
			setAccount(account);
			props.onLogin(account, provider);
		}
	};

	const logout = async () => {
		//logout process
		setAccount(undefined);
		handleClose();
		props.onLogout();
	};

	return (
		<AppBar position="static">
			<Toolbar>
				<IconButton onClick={() => navigate("")}>
					<img src={logo} alt="Logo" style={{ height: 30 }} />
				</IconButton>
				<Typography fontSize={30} component="div" sx={{ flexGrow: 1 }}>
					{uiCustomizations.headerTitle}
				</Typography>
				{error && (
					<IconButton
						title={error}
						aria-label="login"
						aria-controls="menu-appbar"
						onClick={() => login()}
						color="inherit"
					>
						<Error />
					</IconButton>
				)}
				{!isLoggedIn && (
					<IconButton
						size="large"
						title="Login"
						aria-label="login"
						aria-controls="menu-appbar"
						onClick={() => login()}
						color="inherit"
					>
						<Login />
					</IconButton>
				)}
				{isLoggedIn && (
					<>
						<IconButton
							size="small"
							aria-label="account of current user"
							aria-controls="menu-appbar"
							aria-haspopup="true"
							onClick={handleMenu}
							color="inherit"
							style={{ fontSize: 12 }}
							title={account!.address}
						>
							{`${account!.address.slice(0, 7)}...`}{" "}
							<ArrowDownward style={{ marginLeft: 4, width: 12 }} />
						</IconButton>
						<Menu
							id="menu-appbar"
							anchorEl={anchorEl}
							keepMounted
							open={Boolean(anchorEl)}
							onClose={handleClose}
						>
							<MenuItem onClick={logout}>
								<ListItemIcon>
									<Logout fontSize="small" />
								</ListItemIcon>
								<Typography variant="inherit" noWrap>
									Logout
								</Typography>
							</MenuItem>
						</Menu>
					</>
				)}
			</Toolbar>
		</AppBar>
	);
};

type EntrypointsType = Record<string, EntrypointName.Type<string>>;
type EntrypointDisplayNamesType = Record<string, string>;
type EntrypointsUiType = Record<
	string,
	(props: {
		contract: ContractAddress.Type;
		uiSchema?: UiSchema | undefined;
		uiWidgets?: RegistryWidgetsType | undefined;
	}) => React.JSX.Element
>;
type InitType = (props: {
	onInitialize: (contract: ContractAddress.Type) => void;
	uiSchema?: UiSchema;
	uiWidgets?: RegistryWidgetsType;
}) => React.JSX.Element;
type ClientModule = {
	ENTRYPOINTS: EntrypointsType;
	ENTRYPOINT_DISPLAY_NAMES: EntrypointDisplayNamesType;
};
type UiModule = {
	ENTRYPOINTS_UI: EntrypointsUiType;
	init: InitType;
};
const ContractType = (props: {
	contractType: string;
	contracts: Contract[];
	onInitialize: (contract: Contract) => void;
	init: InitType;
	entrypoints: EntrypointsType;
	entrypointDisplayNames: EntrypointDisplayNamesType;
	entrypointUi: EntrypointsUiType;
}) => {
	const {
		contractType,
		entrypoints,
		entrypointDisplayNames,
		entrypointUi,
		init,
		onInitialize,
	} = props;
	return (
		<Routes>
			<Route
				path="init"
				element={init({
					onInitialize: (address: ContractAddress.Type) =>
						onInitialize({
							address: address,
							name: contractType!,
							type: contractType!,
						}),
				})}
			/>
			<Route
				path=":index/:subIndex/*"
				element={<ContractLayout contracts={props.contracts} />}
			>
				<Route
					path="*"
					element={
						<ConcordiumContract
							contractType={contractType!}
							entrypoints={entrypoints}
							entrypointDisplayNames={entrypointDisplayNames}
							entrypointUi={entrypointUi}
						/>
					}
				/>
			</Route>
			<Route path="*" element={<ErrorDisplay text="Not Found" />} />
		</Routes>
	);
};

function Footer() {
	return (
		<Box
			sx={{
				width: "100%",
				height: "60px",
				backgroundColor: "grey.900",
				color: "white",
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
				boxSizing: "border-box",
				mt: "auto",
			}}
		></Box>
	);
}

const ConnectedContent = () => {
	const contractFiles = import.meta.glob([`../../lib/generated/*.ts`]);
	const [contractsState, setContractsState] = useState<
		Record<string, { clientModule?: ClientModule; uiModule?: UiModule }>
	>({});

	const contractTypes: string[] = [];
	Object.entries(contractFiles).forEach(([path]) => {
		const contractName = path.split("/").pop()!.split(".")[0];
		if (!contractTypes.includes(contractName)) {
			contractTypes.push(contractName);
		}
	});

	useEffect(() => {
		Object.entries(contractFiles).forEach(([path, contractFile]) => {
			const contractName = path.split("/").pop()!.split(".")[0];
			if (path.endsWith(".ui.ts") && !contractsState[contractName]?.uiModule) {
				contractFile()
					.then((module) =>
						setContractsState((s) => ({
							...s,
							[contractName]: {
								...s[contractName],
								uiModule: module as UiModule,
							},
						})),
					)
					.catch((e) => {
						console.error("Error loading UI", e);
					});
			} else if (
				path.endsWith(".ts") &&
				!contractsState[contractName]?.clientModule
			) {
				contractFile()
					.then((module) =>
						setContractsState((s) => ({
							...s,
							[contractName]: {
								...s[contractName],
								clientModule: module as ClientModule,
							},
						})),
					)
					.catch((e) => {
						console.error("Error loading entrypoints", e);
					});
			}
		});
	}, [contractFiles, contractsState]);
	const [state, dispatch] = useReducer(reducer, initialState());
	const navigate = useNavigate();

	const onInitClicked = (contractType: string) => {
		navigate(`${contractType}/init`);
	};
	const onContractInitialized = (contract: Contract) => {
		dispatch({ type: ActionTypes.AddContract, contract });
	};
	const onDeleteContract = (contract: Contract) => {
		dispatch({ type: ActionTypes.RemoveContract, contract });
	};

	return (
		<Paper variant="outlined">
			<Routes>
				<Route
					path=""
					element={
						<ContractsList
							contractTypes={contractTypes}
							contracts={state.contracts}
							onDelete={onDeleteContract}
							onInit={onInitClicked}
						/>
					}
				/>
				{contractTypes.map((contractType) => (
					<Route
						path={`${contractType}/*`}
						key={contractType}
						element={
							contractsState[contractType]?.clientModule &&
							contractsState[contractType]?.uiModule ? (
								<ContractType
									key={contractType}
									contractType={contractType}
									contracts={state.contracts.filter(
										(c) => c.type === contractType,
									)}
									onInitialize={onContractInitialized}
									init={contractsState[contractType].uiModule!.init}
									entrypoints={
										contractsState[contractType].clientModule!.ENTRYPOINTS
									}
									entrypointDisplayNames={
										contractsState[contractType].clientModule!
											.ENTRYPOINT_DISPLAY_NAMES
									}
									entrypointUi={
										contractsState[contractType].uiModule!.ENTRYPOINTS_UI
									}
								/>
							) : (
								<ErrorDisplay text={`Contract: ${contractType} not loaded`} />
							)
						}
					/>
				))}

				<Route path="*" element={<ErrorDisplay text="Not Found Path" />} />
			</Routes>
		</Paper>
	);
};

export default function ContractsPage() {
	const navigate = useNavigate();
	const [wallet, setWallet] = useState<{
		wallet: WalletApi;
		account: AccountAddress.Type;
	}>();
	const isLoggedIn = wallet !== undefined;
	const onLogout = () => {
		setWallet(undefined);
		navigate("");
	};

	const DisconnectedContent = () => {
		return InfoDisplay({
			text: "Please connect to Concordium Wallet",
		});
	};
	const theme = useTheme();

	return (
		<Box
			sx={{
				flexGrow: 1,
				backgroundColor: theme.palette.background.default,
				display: "flex",
				flexDirection: "column",
				minHeight: "100%",
			}}
		>
			<ContractsAppBar
				onLogin={(account, wallet) => setWallet({ account, wallet })}
				onLogout={onLogout}
			/>
			<Box
				sx={{
					display: "flex",
					flexDirection: "column",
				}}
			>
				<Box
					sx={{
						padding: 2,
						flexGrow: 1,
						backgroundColor: theme.palette.background.default,
						display: "flex",
						flexDirection: "column",
					}}
				>
					{isLoggedIn ? <ConnectedContent /> : <DisconnectedContent />}
				</Box>
			</Box>
			<Footer />
		</Box>
	);
}
