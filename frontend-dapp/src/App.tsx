import {
	BrowserRouter as Router,
	Route,
	Routes,
	Navigate,
} from "react-router-dom";
import { Box } from "@mui/material";
import ContractsPage from "./components/contracts/ContractsPage";
import ConcordiumWalletProvider from "./components/WalletProvider";
import ConcordiumNodeClientProvider from "./components/NodeClientProvider";
import { createTheme, ThemeProvider } from '@mui/material/styles';
import { themeOptions } from "./config/theme";
import "./App.css"


// Main layout component
function Layout() {

	const theme = createTheme(themeOptions);

	return (
			<ConcordiumNodeClientProvider>
			<ConcordiumWalletProvider>
				<ThemeProvider theme={theme}>				
					<Router>
					<Box
						sx={{
							display: "flex",
							flexDirection: "column",
							height: "100%",
							width: "100%",
							boxSizing: "border-box",
						}}
					>
							<Routes>
								<Route path="contracts/*" Component={ContractsPage} />
								<Route path="*" element={<Navigate to="contracts" replace />} />
							</Routes>
					</Box>

				</Router>
				</ThemeProvider>
			</ConcordiumWalletProvider>
		</ConcordiumNodeClientProvider>
	);
}


export default Layout;