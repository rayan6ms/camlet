import { defineConfig } from "vite";

export default defineConfig({
	root: "src/renderer",
	resolve: {
		tsconfigPaths: true,
	},
	server: {
		host: "127.0.0.1",
		port: 5173,
		strictPort: true,
	},
	build: {
		outDir: "../../dist/renderer",
		emptyOutDir: true,
	},
});
