/* tslint:disable */
/* eslint-disable */

/**
 * Stof Document.
 * This is the entire interface for wasm/js (Runtime + Graph).
 */
export class Stof {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Give this document network access (the Http library, using fetch). Off by default.
     */
    allowHttp(): void;
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
     * Call a singular function in the document (by path).
     * If no arguments, pass undefined as args.
     * Otherwise, pass an array of arguments as args.
     */
    call_with_gate(path: string, args: any, acquire: Function, release: Function): Promise<any>;
    /**
     * Get the ID of this document as a string.
     */
    docid(): string;
    /**
     * Get a value from this graph using the Stof runtime (all language features supported).
     */
    get(path: string, start: any): any;
    /**
     * Insert a JS function as a library function, available in Stof.
     */
    js_library_function(func: StofFunc): void;
    /**
     * Construct a new document.
     */
    constructor();
    /**
     * Import a JS object value.
     */
    objImport(js_obj: any, node: any): boolean;
    /**
     * Parse Stof into this document, optionally within the specified node (pass null for root node).
     */
    parse(stof: string, node: any, profile: string): boolean;
    /**
     * Run functions with the given attribute(s) in this document.
     * Attributes defaults to #[main] functions if null or undefined.
     */
    run_with_gate(attributes: any, acquire: Function, release: Function): Promise<string>;
    /**
     * Set a value onto this graph using the Stof runtime.
     */
    set(path: string, value: any, start: any): boolean;
    /**
     * String export, using a format of choice.
     */
    stringExport(format: string, node: any): string;
    /**
     * String import, using a format of choice (including stof).
     */
    stringImport(src: string, format: string, node: any, profile: string): boolean;
    /**
     * Synchronous call a singular function in the document (by path).
     * If no arguments, pass undefined as args.
     * Otherwise, pass an array of arguments as args.
     * Async TS lib functions will not work with this, but it will be faster.
     */
    sync_call(path: string, args: any): any;
    /**
     * Synchronous run functions with the given attribute(s) in this document.
     * Attributes defaults to #[main] functions if null or undefined.
     * Async TS lib functions will not work with this, but it will be faster.
     */
    sync_run(attributes: any): string;
}

/**
 * JS Library Function.
 */
export class StofFunc {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Doc id for this function.
     */
    docid(): string;
    /**
     * Create a new Stof function from a JS function.
     */
    constructor(docid: string, library: string, name: string, js_function: any, is_async: boolean);
    /**
     * Name this function's parameters, so Stof can call it with named arguments (Ex. `Http.fetch(url, bearer = 'x')`).
     * Parameters are optional: anything not passed arrives as null.
     */
    setParams(names: string[]): void;
}

export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_stof_free: (a: number, b: number) => void;
    readonly __wbg_stoffunc_free: (a: number, b: number) => void;
    readonly start: () => void;
    readonly stof_allowHttp: (a: number) => void;
    readonly stof_binaryExport: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly stof_binaryImport: (a: number, b: any, c: number, d: number, e: any, f: number, g: number) => [number, number, number];
    readonly stof_call_with_gate: (a: number, b: number, c: number, d: any, e: any, f: any) => any;
    readonly stof_docid: (a: number) => [number, number];
    readonly stof_get: (a: number, b: number, c: number, d: any) => any;
    readonly stof_js_library_function: (a: number, b: number) => void;
    readonly stof_new: () => number;
    readonly stof_objImport: (a: number, b: any, c: any) => [number, number, number];
    readonly stof_parse: (a: number, b: number, c: number, d: any, e: number, f: number) => [number, number, number];
    readonly stof_run_with_gate: (a: number, b: any, c: any, d: any) => any;
    readonly stof_set: (a: number, b: number, c: number, d: any, e: any) => number;
    readonly stof_stringExport: (a: number, b: number, c: number, d: any) => [number, number, number, number];
    readonly stof_stringImport: (a: number, b: number, c: number, d: number, e: number, f: any, g: number, h: number) => [number, number, number];
    readonly stof_sync_call: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly stof_sync_run: (a: number, b: any) => [number, number, number, number];
    readonly stoffunc_docid: (a: number) => [number, number];
    readonly stoffunc_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: any, h: number) => number;
    readonly stoffunc_setParams: (a: number, b: number, c: number) => void;
    readonly wasm_bindgen_3b9afeaa82ee37f3___convert__closures_____invoke___js_sys_5d800256f981837c___Function_fn_wasm_bindgen_3b9afeaa82ee37f3___JsValue_____wasm_bindgen_3b9afeaa82ee37f3___sys__Undefined___js_sys_5d800256f981837c___Function_fn_wasm_bindgen_3b9afeaa82ee37f3___JsValue_____wasm_bindgen_3b9afeaa82ee37f3___sys__Undefined_______true_: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_3b9afeaa82ee37f3___convert__closures_____invoke___wasm_bindgen_3b9afeaa82ee37f3___JsValue__core_608f92abc48d28da___result__Result_____wasm_bindgen_3b9afeaa82ee37f3___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
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
