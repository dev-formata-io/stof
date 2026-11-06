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
     * Attributes the html format writes only when allowed: `style`, `action`, and `formaction` (off by default).
     * Documents can't change it. Calling it again adds attributes.
     */
    allowHtmlAttributes(attributes: string[]): void;
    /**
     * Elements the html format may write (Ex. the components your page renders): only these, instead of the
     * standard HTML elements and custom elements. Documents can't change it. Calling it again adds elements.
     */
    allowHtmlTags(tags: string[]): void;
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
     * The seal digest of an object (path, or a type as "<Name>"), with its fields' values when `fields`.
     */
    digest(path: string, fields?: boolean | null): string;
    /**
     * Get the ID of this document as a string.
     */
    docid(): string;
    /**
     * End a task with this error (Ex. the host's call gate failed): `result` or `report` returns it.
     */
    fail(task: StofTask, message: string): void;
    /**
     * Get a value from this graph using the Stof runtime (all language features supported).
     */
    get(path: string, start: any): any;
    /**
     * Insert a JS function as a library function, available in Stof.
     */
    js_library_function(func: StofFunc): void;
    /**
     * Keep each function's source text from now on (off by default), so the readable `stof` export writes
     * functions as Stof. It costs the memory of the text.
     */
    keepSource(keep?: boolean | null): void;
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
     * Print nothing to the console (off by default): no parse warnings, test progress, or pln/err/dbg output.
     * Results and errors still come back (Ex. `test` still returns its report).
     */
    quiet(quiet?: boolean | null): void;
    /**
     * Remove a field, function, or object by path (Ex. "config.staging"), like drop() in Stof.
     * A field that holds an object drops the object too, unless shallow (then only the field is removed).
     * Returns whether anything was removed.
     */
    remove(path: string, shallow: boolean): boolean;
    /**
     * A finished run's report (`start_run`).
     */
    report(task: StofTask): string;
    /**
     * A finished call's result (`start_call`, `start_call_source`).
     */
    result(task: StofTask): any;
    /**
     * Seal an object (path, or a type as "<Name>") and everything under it: its functions can be called but not
     * changed (see `model::seal`). With `fields`, its fields are frozen too. With `expected`, only if its digest is
     * that. With `public_key` and `signature`, only if the digest is signed by that key. Returns the digest.
     * With `removable`, it can still be removed whole. With `reads` (paths, looked up each time the part is entered),
     * its code reaches only itself and those objects; with `hosts`, its Http.fetch reaches only those hosts (see
     * `model::seal::Confine`).
     */
    seal(path: string, fields?: boolean | null, expected?: string | null, public_key?: string | null, signature?: string | null, removable?: boolean | null, reads?: string[] | null, hosts?: string[] | null): string;
    /**
     * How an object (a path, or a type as "<Name>") is sealed, as JSON: { part, fields, removable, confined:
     * [{ reads, hosts }] }, or null when it isn't sealed.
     */
    sealed(path: string): string | undefined;
    /**
     * How long a call can run, in milliseconds, before it fails with an execution timeout (null for no limit).
     * Defaults to 120,000 (2 minutes).
     */
    setMaxExecutionTime(ms?: number | null): void;
    /**
     * How much memory a call can use, in bytes, before it fails with a memory limit error (null for no limit).
     * Defaults to 268,435,456 (256 MB).
     */
    setMaxMemory(bytes?: number | null): void;
    /**
     * Set a value onto this graph using the Stof runtime.
     */
    set(path: string, value: any, start: any): boolean;
    /**
     * Start a call to a function in the document (by path), as a task the host runs with `step` and then
     * `result` (doc.ts `call`). Pass an array of arguments, or undefined for none.
     */
    start_call(path: string, args: any): StofTask;
    /**
     * Start a call to a function from Stof source without adding it to the document, as a task the host runs
     * with `step` and then `result` (doc.ts `callSource`). The source's first function runs with self as the node
     * (an object path or ID, the root by default), and everything the source declares is removed after the call.
     */
    start_call_source(src: string, args: any, node: any): StofTask;
    /**
     * Start running functions with the given attribute(s) in this document (#[main] by default), as a task the
     * host runs with `step` and then `report` (doc.ts `run`).
     */
    start_run(attributes: any): StofTask;
    /**
     * Run one step of a task: 0 when it's finished, 1 for more, 2 for more after yielding to the event loop.
     * A document that's busy (another step holds it) ends the task with that error (returns 0).
     */
    step(task: StofTask): number;
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
     * Synchronous call a function from Stof source without adding it to the document.
     * Async TS lib functions will not work with this, but it will be faster.
     */
    sync_call_source(src: string, args: any, node: any): any;
    /**
     * Synchronous run functions with the given attribute(s) in this document.
     * Attributes defaults to #[main] functions if null or undefined.
     * Async TS lib functions will not work with this, but it will be faster.
     */
    sync_run(attributes: any): string;
    /**
     * Unseal an object that `seal` sealed (path, or a type as "<Name>").
     */
    unseal(path: string): void;
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

/**
 * A call in progress (see `Stof::step`): the host runs it one step at a time, so a trap in the engine ends the
 * call with an error the host can catch.
 */
export class StofTask {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
}

export function start(): void;

/**
 * Edit a Stof text in place, with edits as JSON text (see `parser::edit::edits_from_json`).
 */
export function stofEdit(text: string, edits: string): string;

/**
 * A new secret key (base64, 32 bytes from the system's secure random source).
 */
export function stofGenerateKey(): string;

/**
 * The fields and functions in a Stof text, as JSON (see `parser::edit::outline_json`).
 */
export function stofOutline(text: string): string;

/**
 * The public key (base64) for a secret key.
 */
export function stofPublicKey(secret_key: string): string;

/**
 * Sign a message (a string's UTF-8 bytes, or bytes) under a context with a secret key: the signature (base64).
 */
export function stofSign(secret_key: string, context: string, message: any): string;

/**
 * Is this a signature over this message (a string or bytes), under this context, by this public key?
 */
export function stofVerify(public_key: string, context: string, message: any, signature: string): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_stof_free: (a: number, b: number) => void;
    readonly __wbg_stoffunc_free: (a: number, b: number) => void;
    readonly __wbg_stoftask_free: (a: number, b: number) => void;
    readonly start: () => void;
    readonly stofEdit: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly stofGenerateKey: () => [number, number];
    readonly stofOutline: (a: number, b: number) => [number, number, number, number];
    readonly stofPublicKey: (a: number, b: number) => [number, number, number, number];
    readonly stofSign: (a: number, b: number, c: number, d: number, e: any) => [number, number, number, number];
    readonly stofVerify: (a: number, b: number, c: number, d: number, e: any, f: number, g: number) => number;
    readonly stof_allowHtmlAttributes: (a: number, b: number, c: number) => [number, number];
    readonly stof_allowHtmlTags: (a: number, b: number, c: number) => [number, number];
    readonly stof_allowHttp: (a: number) => [number, number];
    readonly stof_binaryExport: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly stof_binaryImport: (a: number, b: any, c: number, d: number, e: any, f: number, g: number) => [number, number, number];
    readonly stof_digest: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly stof_docid: (a: number) => [number, number];
    readonly stof_fail: (a: number, b: number, c: number, d: number) => void;
    readonly stof_get: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly stof_js_library_function: (a: number, b: number) => [number, number];
    readonly stof_keepSource: (a: number, b: number) => [number, number];
    readonly stof_new: () => number;
    readonly stof_objImport: (a: number, b: any, c: any) => [number, number, number];
    readonly stof_parse: (a: number, b: number, c: number, d: any, e: number, f: number) => [number, number, number];
    readonly stof_quiet: (a: number, b: number) => [number, number];
    readonly stof_remove: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly stof_report: (a: number, b: number) => [number, number, number, number];
    readonly stof_result: (a: number, b: number) => [number, number, number];
    readonly stof_seal: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => [number, number, number, number];
    readonly stof_sealed: (a: number, b: number, c: number) => [number, number, number, number];
    readonly stof_set: (a: number, b: number, c: number, d: any, e: any) => [number, number, number];
    readonly stof_setMaxExecutionTime: (a: number, b: number, c: number) => [number, number];
    readonly stof_setMaxMemory: (a: number, b: number, c: number) => [number, number];
    readonly stof_start_call: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly stof_start_call_source: (a: number, b: number, c: number, d: any, e: any) => [number, number, number];
    readonly stof_start_run: (a: number, b: any) => [number, number, number];
    readonly stof_step: (a: number, b: number) => number;
    readonly stof_stringExport: (a: number, b: number, c: number, d: any) => [number, number, number, number];
    readonly stof_stringImport: (a: number, b: number, c: number, d: number, e: number, f: any, g: number, h: number) => [number, number, number];
    readonly stof_sync_call: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly stof_sync_call_source: (a: number, b: number, c: number, d: any, e: any) => [number, number, number];
    readonly stof_sync_run: (a: number, b: any) => [number, number, number, number];
    readonly stof_unseal: (a: number, b: number, c: number) => [number, number];
    readonly stoffunc_docid: (a: number) => [number, number];
    readonly stoffunc_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: any, h: number) => number;
    readonly stoffunc_setParams: (a: number, b: number, c: number) => void;
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
