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


function Footer() {
	return (
		<Box
			sx={{
				width: "100%", // full width
				height: "60px", // fixed height
				backgroundColor: "grey.900", // dark background color
				color: "white", // white text color
				position: "fixed", // fixed position
				bottom: 0, // stick to bottom
				display: "flex", // use flexbox for centering content
				alignItems: "center", // center content vertically
				justifyContent: "center", // center content horizontally
				zIndex: (theme) => theme.zIndex.drawer + 1,
				boxSizing: "border-box", // include padding in width/height
			}}
		>
			{/* Footer content goes here */}
		</Box>
	);
}

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
							minHeight: "100vh",
							width: "100%",
							boxSizing: "border-box",
						}}
					>
						<Box pb="70px" sx={{ flex: 1 }}>
							<Routes>
								<Route path="contracts/*" Component={ContractsPage} />
								<Route path="*" element={<Navigate to="contracts" replace />} />
							</Routes>
						</Box>
						<Footer />
					</Box>
				</Router>
				</ThemeProvider>
			</ConcordiumWalletProvider>
		</ConcordiumNodeClientProvider>
	);
}


export default Layout;