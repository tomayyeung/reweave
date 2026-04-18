import init, * as wasm from "@wasm/frontend";

let wasmReady: Promise<typeof wasm> | null = null;

export function getWasm() {
  if (!wasmReady) {
    wasmReady = init().then(() => wasm);
  }
  return wasmReady;
}