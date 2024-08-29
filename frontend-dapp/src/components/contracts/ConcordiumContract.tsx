import { Stack, Grid, Paper, Typography } from "@mui/material";
import { Route, Routes, useLocation, useParams } from "react-router-dom";
import { ContractAddress, EntrypointName } from "@concordium/web-sdk";
import EntrypointsList from "./EntrypointsList";
import ContractBreadcrumb from "./common/ContractBreadcrumb";

export default function ConcordiumContract(props: {
	contractType: string;
	entrypoints: Record<string, EntrypointName.Type<string>>;
	entrypointDisplayNames: Record<string, string>;
	entrypointUi: Record<
		string,
		(props: { contract: ContractAddress.Type }) => JSX.Element
	>;
}) {
	const { index, subIndex } = useParams();
	const contract = ContractAddress.create(BigInt(index!), BigInt(subIndex!));
	const { contractType, entrypoints, entrypointDisplayNames, entrypointUi } =
		props;
	const { pathname } = useLocation();
	const paths = pathname.split("/");
	const path = paths[paths.length - 1];

	return (
		<Stack>
			<ContractBreadcrumb
				contractType={contractType}
				index={index!}
				subIndex={subIndex!}
				entrypointDisplayNames={entrypointDisplayNames}
				path={path}
			/>
			<Grid container spacing={1}>
				<Grid item xs={12} md={9}>
					<Paper variant="outlined" sx={{ border: 0 }}>
						<Routes>
							{Object.keys(entrypoints).map((entrypoint) => (
								<Route
									key={entrypoint}
									path={entrypoint}
									element={entrypointUi[entrypoint]({ contract })}
								/>
							))}
							<Route
								path=""
								element={
									<Typography variant="h3">Select an Entrypoint</Typography>
								}
							/>
							<Route
								path="*"
								element={
									<Typography variant="h3">Unknown Entrypoint</Typography>
								}
							/>
						</Routes>
					</Paper>
				</Grid>
				<Grid item xs={12} md={3}>
					<Paper>
						<EntrypointsList
							entrypoints={entrypoints}
							entrypointDisplayNames={entrypointDisplayNames}
							selectedPath={path}
						/>
					</Paper>
				</Grid>
			</Grid>
		</Stack>
	);
}
