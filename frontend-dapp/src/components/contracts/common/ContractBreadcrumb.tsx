import { Breadcrumbs, useTheme } from "@mui/material";
import { capitalCase } from "change-case";
import { Link } from "react-router-dom";

export default function ContractBreadcrumb(props: {
	contractType: string;
	index: string;
	subIndex: string;
	entrypointDisplayNames: Record<string, string>;
	path?: string;
}) {
	const { index, subIndex, path = "" } = props;
	const theme = useTheme();

	return (
		<Breadcrumbs>
			<Link style={{ textDecoration: "none" }} to="/contracts">
				<span
					style={{
						color: theme.palette.text.secondary,
					}}
				>
					Contracts
				</span>
			</Link>
			<Link
				style={{ textDecoration: "none" }}
				to={`/contracts/${props.contractType}/${index}/${subIndex}`}
			>
				<span
					style={{
						color: theme.palette.text.secondary,
					}}
				>
					{`${capitalCase(props.contractType)} <${index},${subIndex}>`}
				</span>
			</Link>
			{props.entrypointDisplayNames[path] && (
				<Link
					style={{ textDecoration: "none" }}
					to={`/contracts/${props.contractType}/${index}/${subIndex}/${path}`}
				>
					<span
						style={{
							color: theme.palette.text.secondary,
						}}
					>
						{props.entrypointDisplayNames[path]}
					</span>
				</Link>
			)}
		</Breadcrumbs>
	);
}
