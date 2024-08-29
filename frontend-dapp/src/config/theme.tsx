import { ThemeOptions } from "@mui/material/styles";

export const themeOptions: ThemeOptions = {
	palette: {
		mode: "dark",
		primary: {
			main: "#0f0",
		},
		background: {
			default: "#111111",
			paper: "#212121",
		},
		text: {
			primary: "#ffffff",
			secondary: "#a5d6a7",
		},
	},
	spacing: 8,
	shape: {
		borderRadius: 4,
	},
	components: {
		MuiButtonBase: {
			defaultProps: {
				disableRipple: true,
			},
		},
		MuiLink: {
			styleOverrides: {
				root: {
					color: "#0f0",
					textDecoration: "none",
					"&:hover": {
						textDecoration: "underline",
					},
					"&:active": {
						color: "#ffffff",
						textDecoration: "underline",
					},
					"&:visited": {
						color: "#a5d6a7",
					},
				},
			},
		},
		MuiBreadcrumbs: {
			styleOverrides: {
				root: {
					color: "#a5d6a7",
				},
				separator: {
					color: "#ffffff",
				},
				ol: {
					alignItems: "center",
				},
			},
		},
	},
};

export const uiCustomizations = {
	headerTitle: "AdminPannel",
};
