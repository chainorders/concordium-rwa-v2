import { Alert } from "@mui/material";

export default function InfoDisplay(props: { text: string }) {
	return props.text ? <Alert severity="info">{props.text}</Alert> : <></>;
}
