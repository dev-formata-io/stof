/* tslint:disable */
/* eslint-disable */

export class Stof {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Binary export (Uint8Array), using a format of choice.
   * Format can also be a content type (for HTTP-like situations).
   */
  binaryExport(format: string, node: any): any;
  /**
   * Binary import (Uint8Array), using a format of choice.
   * Format can also be a content type (for HTTP-like situations).
   */
  binaryImport(bytes: any, format: string, node: any, profile: string): boolean;
  /**
   * Import a JS object value.
   */
  objImport(js_obj: any, node: any): boolean;
  /**
   * String export, using a format of choice.
   */
  stringExport(format: string, node: any): string;
  /**
   * String import, using a format of choice (including stof).
   */
  stringImport(src: string, format: string, node: any, profile: string): boolean;
  /**
   * Insert a JS function as a library function, available in Stof.
   */
  js_library_function(func: StofFunc): void;
  /**
   * Get a value from this graph using the Stof runtime (all language features supported).
   */
  get(path: string, start: any): any;
  /**
   * Construct a new document.
   */
  constructor();
  /**
   * Run functions with the given attribute(s) in this document.
   * Attributes defaults to #[main] functions if null or undefined.
   */
  run(attributes: any): Promise<string>;
  /**
   * Set a value onto this graph using the Stof runtime.
   */
  set(path: string, value: any, start: any): boolean;
  /**
   * Call a singular function in the document (by path).
   * If no arguments, pass undefined as args.
   * Otherwise, pass an array of arguments as args.
   */
  call(path: string, args: any): Promise<any>;
  /**
   * Parse Stof into this document, optionally within the specified node (pass null for root node).
   */
  parse(stof: string, node: any, profile: string): boolean;
}

export class StofFunc {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create a new Stof function from a JS function.
   */
  constructor(library: string, name: string, js_function: any, is_async: boolean);
}

export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_stof_free: (a: number, b: number) => void;
  readonly start: () => void;
  readonly stof_binaryExport: (a: number, b: number, c: number, d: any) => [number, number, number];
  readonly stof_binaryImport: (a: number, b: any, c: number, d: number, e: any, f: number, g: number) => [number, number, number];
  readonly stof_call: (a: number, b: number, c: number, d: any) => any;
  readonly stof_get: (a: number, b: number, c: number, d: any) => any;
  readonly stof_js_library_function: (a: number, b: number) => void;
  readonly stof_new: () => number;
  readonly stof_objImport: (a: number, b: any, c: any) => [number, number, number];
  readonly stof_parse: (a: number, b: number, c: number, d: any, e: number, f: number) => [number, number, number];
  readonly stof_run: (a: number, b: any) => any;
  readonly stof_set: (a: number, b: number, c: number, d: any, e: any) => number;
  readonly stof_stringExport: (a: number, b: number, c: number, d: any) => [number, number, number, number];
  readonly stof_stringImport: (a: number, b: number, c: number, d: number, e: number, f: any, g: number, h: number) => [number, number, number];
  readonly __wbg_stoffunc_free: (a: number, b: number) => void;
  readonly stoffunc_new: (a: number, b: number, c: number, d: number, e: any, f: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__h53d5cf04cab8438f: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h72b14ab7db8750ca: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__ha84735728bfe97a9: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
