
import { load as loadFfi } from "./xcelerate_core-ffi.js";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Smart Loading: Select the correct native library for the current platform
let libName;
const platform = os.platform();

if (platform === "win32") {
    libName = "xcelerate_core.dll";
} else if (platform === "darwin") {
    libName = "libxcelerate_core.dylib";
} else {
    libName = "libxcelerate_core.so";
}

const libPath = path.join(__dirname, libName);
loadFfi(libPath);


export * from "./xcelerate_core.js";