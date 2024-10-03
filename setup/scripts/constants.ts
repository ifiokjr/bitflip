import path from "node:path";

export const EXTENSIONS_FOLDER = path.join(
	import.meta.dirname ?? "",
	"extensions",
);

export const EXTENSIONS = {
	phantom: {
		id: "bfnaelmomeimhlpmgjnjophhpkkoljpa",
		version: "24.10.0",
		path: path.join(EXTENSIONS_FOLDER, "phantom"),
	},
	solflare: {
		id: "bhhhlbepdkbapadjdnnojkbgioiodbic",
		version: "1.69.0",
		path: path.join(EXTENSIONS_FOLDER, "solflare"),
	},
	backpack: {
		id: "aflkmfhebedbjioipglgcbcmnbpgliof",
		version: "0.10.74",
		path: path.join(EXTENSIONS_FOLDER, "backpack"),
	},
};
