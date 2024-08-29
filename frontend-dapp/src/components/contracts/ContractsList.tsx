import {
	IconButton,
	List,
	ListItem,
	ListItemButton,
	ListItemIcon,
	ListItemText,
	ListSubheader,
	Typography,
} from "@mui/material";
import { Contract } from "./ContractTypes";
import {
	AddBoxOutlined,
	CodeRounded,
	Delete,
	OpenInBrowser,
} from "@mui/icons-material";
import { Link } from "react-router-dom";
import { ReactNode } from "react";
import { useTheme } from "@mui/material/styles";

function ContractLink(props: { contract: Contract }) {
	const theme = useTheme();

	const contractAddressString = `${props.contract.address.index.toString()}/${props.contract.address.subindex.toString()}`;
	return (
		<Link
			id="id"
			to={`${props.contract.type}/${contractAddressString}`}
			target="_self"
			style={{ textDecoration: "none", width: "80%" }}
		>
			<span
				style={{
					color: theme.palette.text.primary,
					width: "80%",
				}}
			>
				{props.contract.name} ({contractAddressString})
			</span>
		</Link>
	);
}

const InitListItem = (props: { onClick: () => void; children: ReactNode }) => {
	return (
		<ListItem
			secondaryAction={
				<IconButton edge="end" aria-label="add" onClick={() => props.onClick()}>
					<AddBoxOutlined />
				</IconButton>
			}
		>
			<ListItemText
				primary={<Typography align="right">{props.children}</Typography>}
			/>
		</ListItem>
	);
};

const ContractGroup = (props: {
	contractType: string;
	contracts: Contract[];
	onDelete?: (contract: Contract) => void;
	onInit?: (name: string) => void;
}) => {
	return (
		<>
			<ListSubheader>{props.contractType}</ListSubheader>
			{props.contracts.map((contract) => {
				return (
					<div style={{ display: "flex", flexDirection: "row" }}>
						<ListItem key={contract.address.index.toString()}>
							<ListItemButton>
								<ListItemIcon>
									<CodeRounded />
								</ListItemIcon>
								<ListItemText primary={<ContractLink contract={contract} />} />
							</ListItemButton>
						</ListItem>
						<div
							style={{
								display: "flex",
								flexDirection: "row",
								justifyContent: "center",
								alignItems: "center",
								marginRight: 15,
								gap: 5,
							}}
						>
							<Link
								to={`${contract.type}/${contract.address.index.toString()}/${contract.address.subindex.toString()}`}
								target="_blank"
							>
								<IconButton edge="end" aria-label="open contract">
									<OpenInBrowser />
								</IconButton>
							</Link>

							{props.onDelete && (
								<IconButton
									edge="end"
									aria-label="delete"
									onClick={() => props.onDelete!(contract)}
								>
									<Delete />
								</IconButton>
							)}
						</div>
					</div>
				);
			})}
			{props.onInit && (
				<InitListItem onClick={() => props.onInit!(props.contractType)}>
					Initialize new {props.contractType}
				</InitListItem>
			)}
		</>
	);
};

interface Props {
	contractTypes: string[];
	contracts: Contract[];
	onDelete?: (contract: Contract) => void;
	onInit?: (name: string) => void;
}
export default function ContractsList(props: Props) {
	return (
		<List>
			{props.contractTypes.map((contractType) => {
				return (
					<ContractGroup
						key={contractType}
						contracts={props.contracts
							.filter((c) => !!c && !!c.type)
							.filter((c) => c.type === contractType)}
						onDelete={props.onDelete}
						onInit={props.onInit}
						contractType={contractType}
					/>
				);
			})}
		</List>
	);
}
