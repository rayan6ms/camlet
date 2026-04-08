import { rmSync } from "node:fs";
import path from "node:path";

const projectRoot = process.cwd();
const targets = [
	"dist",
	"dist-electron",
	".tsbuildinfo",
	"coverage",
	"release",
].map((target) => path.join(projectRoot, target));

for (const target of targets) {
	rmSync(target, { force: true, recursive: true });
}
