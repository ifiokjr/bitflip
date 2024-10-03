import typography from "@tailwindcss/typography";
import animated from "tailwindcss-animated";

/**
 * @param {string[]} files
 * @returns {import('tailwindcss').Config}
 */
export function setupTailwind(files) {
	return {
		content: {
			relative: true,
			files: [...files],
		},
		darkMode: ["selector", '[data-theme="dark"]'],
		theme: {
			extend: {
				fontFamily: {
					sans: [
						"Short Stack",
						"ui-sans-serif",
						"system-ui",
						"sans-serif",
						"Apple Color Emoji",
						"Segoe UI Emoji",
						"Segoe UI Symbol",
						"Noto Color Emoji",
					],
				},
			},
		},
		plugins: [typography, animated],
		daisyui: {
			themes: false,
			darkTheme: "dark",
		},
	};
}
