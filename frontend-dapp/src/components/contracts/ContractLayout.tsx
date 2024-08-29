import { Grid, Paper, Stack, Typography, Box } from "@mui/material";
import { Contract } from "./ContractTypes";
import { Outlet, useParams } from "react-router-dom";
import { useEffect, useState } from "react";
import concordiumNodeClient from "../../lib/ConcordiumNodeClient";
import { ContractAddress, InstanceInfo } from "@concordium/web-sdk";
import CCDScanModuleLink from "../common/concordium/CCDScanModuleLink";
import CCDScanContractLink from "../common/concordium/CCDScanContractLink";
import CCDScanAccountLink from "../common/concordium/CCDScanAccountLink";

export default function ContractLayout(props: { contracts: Contract[] }) {
	const { index, subIndex } = useParams();
	const contract = props.contracts.find((contract) => {
		return (
			contract.address.index.toString() === index &&
			contract.address.subindex.toString() === subIndex
		);
	});

	const [onChainInfo, setOncChainInfo] = useState<InstanceInfo>();
	useEffect(() => {
		if (onChainInfo) {
			return;
		}

		concordiumNodeClient
			.getInstanceInfo(
				ContractAddress.create(BigInt(index!), BigInt(subIndex!)),
			)
			.then((info) => {
				setOncChainInfo(info);
			});
	});

	if (!contract) {
		return (
			<div>
				<h1>
					{index}/{subIndex}
					<Typography variant="body1" color="error">
						Contract not found
					</Typography>
				</h1>
			</div>
		);
	}

	return (
		<Stack spacing={2}>
			<Grid container spacing={1}>
				<Grid item xs={12} md={8}>
					<Outlet context={contract} />
				</Grid>
				<Grid item xs={0} md={4}>
					<Box sx={{ py: 3, width: "90%" }}>
						<Paper
							sx={{
								p: 2,
								mb: 2,
								overflow: "hidden",
								textOverflow: "ellipsis",
								wordWrap: "break-word",
							}}
						>
							<Typography variant="h4" fontSize={18} gutterBottom>
								On Chain Information
							</Typography>
							{onChainInfo && (
								<>
									<Typography variant="body1" gutterBottom>
										Module Ref:
										<CCDScanModuleLink
											moduleRef={onChainInfo?.sourceModule.moduleRef}
										/>
									</Typography>
									<Typography variant="body1" gutterBottom>
										Name:
										<CCDScanContractLink
											text={onChainInfo?.name.value}
											index={contract.address.index.toString()}
											subIndex={contract.address.subindex.toString()}
										/>
									</Typography>
									<Typography variant="body1">
										Owner:
										<CCDScanAccountLink account={onChainInfo?.owner.address} />
									</Typography>
								</>
							)}
						</Paper>

						<Paper
							sx={{
								p: 2,
								mb: 2,
								overflow: "hidden",
								textOverflow: "ellipsis",
								wordWrap: "break-word",
							}}
						>
							<Typography variant="h4" fontSize={18} gutterBottom>
								Module Actions
							</Typography>
							{onChainInfo && (
								<>
									{onChainInfo.methods.map((method) => (
										<Typography key={method.value} variant="body2" gutterBottom>
											{method.value}
										</Typography>
									))}
								</>
							)}
						</Paper>
					</Box>
				</Grid>
			</Grid>
		</Stack>
	);
}
