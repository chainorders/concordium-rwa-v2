import { List, ListItem, ListItemButton, ListItemText } from "@mui/material";
import { useNavigate } from "react-router-dom";
import { EntrypointName } from "@concordium/web-sdk";

export default function EntrypointsList(props: {
	entrypoints: Record<string, EntrypointName.Type<string>>;
	entrypointDisplayNames: Record<string, string>;
	disabled?: boolean;
	selectedPath?: string;
}) {
	const navigation = useNavigate();
	return (
		<List>
			{Object.keys(props.entrypoints).map((key) => {
				return (
					<ListItem disablePadding key={key} disabled={props.disabled}>
						<ListItemButton
							onClick={() => navigation(key)}
							disabled={props.disabled}
							selected={props.selectedPath === key}
						>
							<ListItemText
								primary={props.entrypointDisplayNames[key]}
								secondary={key}
							/>
						</ListItemButton>
					</ListItem>
				);
			})}
		</List>
	);
}
