import { Alert } from "@mui/material";

export default function ErrorDisplay(props: { text: string }) {
	return props.text ? <Alert severity="error">{props.text}</Alert> : <></>;
}
