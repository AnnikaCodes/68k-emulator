declare namespace wasm_bindgen {
	/* tslint:disable */
	/* eslint-disable */
	/**
	*/
	export function main(): void;
	/**
	*/
	export class REPLBackend {
	  free(): void;
	/**
	* @returns {REPLBackend}
	*/
	  static new(): REPLBackend;
	/**
	* @param {string} assembly
	* @returns {string}
	*/
	  interpret_assembly(assembly: string): string;
	}
	
}

declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_replbackend_free: (a: number) => void;
  readonly replbackend_new: () => number;
  readonly replbackend_interpret_assembly: (a: number, b: number, c: number, d: number) => void;
  readonly main: () => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_start: () => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
declare function wasm_bindgen (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
