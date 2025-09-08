/* tslint:disable */
/* eslint-disable */
export function start(): void;
/**
 * Stof Document.
 * This is the entire interface for wasm/js (Runtime + Graph).
 */
export class Stof {
  free(): void;
  /**
   * Construct a new document.
   */
  constructor();
  /**
   * Run functions with the given attribute(s) in this document.
   * Attributes defaults to #[main] functions if null or undefined.
   */
  run(attributes: any): string;
  /**
   * Call a singular function in the document (by path).
   * If no arguments, pass undefined as args.
   * Otherwise, pass an array of arguments as args.
   */
  call(path: string, args: any): any;
  /**
   * Insert a JS function as a library function, available in Stof.
   */
  js_library_function(func: StofFunc): void;
  /**
   * Parse Stof into this document, optionally within the specified node (pass null for root node).
   */
  parse(stof: string, node: any): boolean;
  /**
   * Import a JS object value.
   */
  objImport(js_obj: any, node: any): boolean;
  /**
   * String import, using a format of choice (including stof).
   */
  stringImport(src: string, format: string, node: any): boolean;
  /**
   * Binary import (Uint8Array), using a format of choice.
   * Format can also be a content type (for HTTP-like situations).
   */
  binaryImport(bytes: any, format: string, node: any): boolean;
  /**
   * String export, using a format of choice.
   */
  stringExport(format: string, node: any): string;
  /**
   * Binary export (Uint8Array), using a format of choice.
   * Format can also be a content type (for HTTP-like situations).
   */
  binaryExport(format: string, node: any): any;
}
/**
 * JS Library Function.
 */
export class StofFunc {
  free(): void;
  /**
   * Create a new Stof function from a JS function.
   */
  constructor(library: string, name: string, js_function: any);
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_stoffunc_free: (a: number, b: number) => void;
  readonly stoffunc_new: (a: number, b: number, c: number, d: number, e: any) => number;
  readonly start: () => void;
  readonly __wbg_stof_free: (a: number, b: number) => void;
  readonly stof_new: () => number;
  readonly stof_run: (a: number, b: any) => [number, number, number, number];
  readonly stof_call: (a: number, b: number, c: number, d: any) => [number, number, number];
  readonly stof_js_library_function: (a: number, b: number) => void;
  readonly stof_parse: (a: number, b: number, c: number, d: any) => [number, number, number];
  readonly stof_objImport: (a: number, b: any, c: any) => [number, number, number];
  readonly stof_stringImport: (a: number, b: number, c: number, d: number, e: number, f: any) => [number, number, number];
  readonly stof_binaryImport: (a: number, b: any, c: number, d: number, e: any) => [number, number, number];
  readonly stof_stringExport: (a: number, b: number, c: number, d: any) => [number, number, number, number];
  readonly stof_binaryExport: (a: number, b: number, c: number, d: any) => [number, number, number];
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
