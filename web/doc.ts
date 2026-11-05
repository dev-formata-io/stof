//
// Copyright 2025 Formata, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

// @deno-types="./pkg/stof.d.ts"
import init, { Stof, StofFunc, StofTask, stofOutline, stofEdit, stofSign, stofVerify, stofPublicKey } from './pkg/stof.js';
// @deno-types="./pkg/stof.d.ts"
export * from './pkg/stof.js';


/**
 * Auto-initialize Stof WASM for any environment.
 * Detects Node.js vs Browser and loads the WASM file appropriately.
 */
export async function initStof(wasmSource?: string | BufferSource): Promise<unknown> {
    if (StofDoc['initialized']) {
        return StofDoc['initialized'];
    }

    let wasmData: BufferSource | undefined;

    if (wasmSource) {
        // User provided WASM source
        if (typeof wasmSource === 'string') {
            // Check environment using the same detection as auto-load
            // @ts-ignore
            const isDeno = typeof Deno !== 'undefined' && typeof Deno.version !== 'undefined';
            // @ts-ignore
            const isNode = !isDeno && typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

            if (isNode) {
                // It's a file path (Node.js only)
                // @ts-ignore - Node.js module, conditionally imported
                const fs = await import('fs');
                wasmData = fs.readFileSync(wasmSource);
            } else {
                // Browser/Deno: treat string as URL and fetch it
                const response = await fetch(wasmSource);
                wasmData = await response.arrayBuffer();
            }
        } else {
            wasmData = wasmSource;
        }
    } else {
        // Auto-detect environment and load
        // Check for Deno first (it has both process and Deno global)
        // @ts-ignore - Deno global
        const isDeno = typeof Deno !== 'undefined' && typeof Deno.version !== 'undefined';
        
        // Then check for Node.js
        // @ts-ignore - process is Node.js global
        const isNode = !isDeno && typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

        if (isNode) {
            // @ts-ignore - Node.js modules, conditionally imported
            const fs = await import('fs');
            // @ts-ignore
            const { fileURLToPath } = await import('url');
            // @ts-ignore
            const { dirname, join } = await import('path');
            
            const __filename = fileURLToPath(import.meta.url);
            const __dirname = dirname(__filename);
            const wasmPath = join(__dirname, './pkg/stof_bg.wasm');
            wasmData = fs.readFileSync(wasmPath);
        } else {
            // Browser/Deno environment - use fetch with import.meta.url
            const wasmUrl = new URL('./pkg/stof_bg.wasm', import.meta.url);
            const response = await fetch(wasmUrl);
            wasmData = await response.arrayBuffer();
        }
    }

    return await StofDoc.initialize(wasmData);
}


/**
 * Check if Stof WASM is initialized.
 */
export function isStofInitialized(): boolean {
    return StofDoc['initialized'] !== undefined;
}


/**
 * Template function for a document.
 * Stof must be initialized before with `await Doc.initialize()`.
 */
export function stof(strings: TemplateStringsArray, ...values: unknown[]): StofDoc {
    if (!StofDoc['initialized']) {
        throw new Error('Stof not initialized. Call await initStof() first.');
    }
    const doc = new StofDoc();
    let result = '';
    for (let i = 0; i < strings.length; i++) {
        result += strings[i];
        if (i < values.length) result += values[i];
    }
    doc.parse(result);
    return doc;
}


/**
 * Async template function that auto-initializes.
 */
export async function stofAsync(strings: TemplateStringsArray, ...values: unknown[]): Promise<StofDoc> {
    await initStof();
    const doc = new StofDoc();
    let result = '';
    for (let i = 0; i < strings.length; i++) {
        result += strings[i];
        if (i < values.length) result += values[i];
    }
    doc.parse(result);
    return doc;
}


/**
 * Internal mutex for single linear WASM.
 * Prevents race conditions and memory issues.
 */
class WasmMutex {
    private locked = false;
    private waiters: (() => void)[] = [];

    acquire(): Promise<void> {
        if (!this.locked) {
            this.locked = true;
            return Promise.resolve();
        }
        return new Promise<void>((resolve) => {
            this.waiters.push(resolve);
        });
    }

    release(): void {
        if (this.waiters.length > 0) {
            const next = this.waiters.shift()!;
            setTimeout(next, 0); // Resolve on next so frame completes
        } else {
            this.locked = false;
        }
    }
}


/**
 * The engine trapped (Ex. ran out of memory, or an internal error): WebAssembly stopped in the middle of a call,
 * so that document's state can't be trusted. Other documents keep working; this one only throws this error.
 */
function crashedError(reason: string): Error {
    return new Error(`the Stof document crashed (${reason}) and can't be used again: create a new document`);
}

/**
 * The document's engine, with every call into WebAssembly checked: a trap marks the document crashed and throws
 * an error saying so (instead of "busy" forever, since the trap left it mid-call), and so does every later call.
 */
function guardStof(stof: Stof, doc: { crashed: string | null }): Stof {
    const guarded = new Map<PropertyKey, unknown>();
    return new Proxy(stof, {
        get(target, prop, receiver) {
            const value = Reflect.get(target, prop, receiver);
            if (typeof value !== 'function') return value;
            let fn = guarded.get(prop) as Function | undefined;
            if (!fn) {
                fn = function (...args: unknown[]) {
                    if (doc.crashed !== null) {
                        if (prop === 'free') return; // disposing a crashed document is fine
                        throw crashedError(doc.crashed);
                    }
                    try {
                        return (value as Function).apply(target, args);
                    } catch (error) {
                        if (error instanceof WebAssembly.RuntimeError) {
                            doc.crashed = error.message;
                            throw crashedError(error.message);
                        }
                        throw error;
                    }
                };
                guarded.set(prop, fn);
            }
            return fn;
        },
    });
}

/** Yield to the event loop's macrotask queue, so host promises and timers run. */
function yieldToEventLoop(): Promise<void> {
    if (typeof setTimeout !== 'function') return Promise.resolve();
    return new Promise((resolve) => setTimeout(resolve, 0));
}


type StofHeaders = Map<string, unknown> | Record<string, unknown> | null | undefined;

/**
 * Is this URL's host allowed (any host when there's no list)?
 * An entry is a host name (any port, Ex. "api.example.com") or a host and port ("localhost:8080").
 */
function hostAllowed(url: URL, allowed: Set<string> | null): boolean {
    if (!allowed) return true;
    const host = url.hostname.toLowerCase();
    const port = url.port || (url.protocol === 'https:' ? '443' : url.protocol === 'http:' ? '80' : '');
    return allowed.has(host) || allowed.has(`${host}:${port}`);
}

/**
 * Is this IPv4 address (four numbers) on this machine, a private network, or link-local (cloud metadata)?
 */
function privateIPv4(parts: number[]): boolean {
    const [a, b, c] = parts;
    return a === 0 || a === 10 || a === 127 || a >= 224                 // this network, private, loopback, multicast/reserved
        || (a === 169 && b === 254) || (a === 172 && b >= 16 && b < 32) // link-local, private
        || (a === 192 && b === 168) || (a === 100 && b >= 64 && b < 128) // private, carrier-grade NAT
        || (a === 192 && b === 0 && c === 0);                           // IETF protocol assignments
}

/**
 * Is this URL host (as the URL parser writes it: dotted IPv4, bracketed IPv6, or a name) on this machine or a
 * private network? Names are checked as written (`localhost`, `*.local`, `*.internal`, `metadata`, ...): what a
 * name resolves to can't be checked from JavaScript, so a host that must be sure lists the hosts it allows.
 */
function privateHost(host: string): boolean {
    host = host.toLowerCase().replace(/\.$/, '');
    const v4 = /^(\d+)\.(\d+)\.(\d+)\.(\d+)$/.exec(host);
    if (v4) return privateIPv4(v4.slice(1).map(Number));
    if (host.startsWith('[')) {
        const groups = ipv6Groups(host.slice(1, -1));
        if (!groups) return true; // unreadable: refuse
        if (groups.slice(0, 5).every((g) => g === 0) && groups[5] === 0xffff) {
            return privateIPv4([groups[6] >> 8, groups[6] & 0xff, groups[7] >> 8, groups[7] & 0xff]); // ::ffff:a.b.c.d
        }
        const first = groups[0];
        return groups.every((g, i) => g === (i === 7 ? 1 : 0)) // ::1
            || groups.every((g) => g === 0)                      // ::
            || (first & 0xfe00) === 0xfc00                       // unique local
            || (first & 0xffc0) === 0xfe80;                      // link-local
    }
    return host === 'localhost' || host === 'metadata' || host === 'internal'
        || host.endsWith('.localhost') || host.endsWith('.local') || host.endsWith('.internal');
}

/** The eight 16-bit groups of an IPv6 address (the URL parser writes it in hex, without an IPv4 tail). */
function ipv6Groups(address: string): number[] | null {
    const [head, tail] = address.includes('::') ? address.split('::') : [address, null];
    const parse = (part: string) => (part ? part.split(':').map((g) => parseInt(g, 16)) : []);
    const left = parse(head);
    const right = tail === null ? [] : parse(tail);
    const groups = tail === null ? left : [...left, ...new Array(8 - left.length - right.length).fill(0), ...right];
    return groups.length === 8 && groups.every((g) => Number.isInteger(g) && g >= 0 && g <= 0xffff) ? groups : null;
}

/** May a request go to this URL? Throws the reason when it may not. */
function checkHost(url: URL, allowed: Set<string> | null, privateNetwork: boolean, redirect: boolean): void {
    const what = redirect ? `redirected to ${url.href}, which` : url.origin;
    if (allowed) {
        if (!hostAllowed(url, allowed)) throw new Error(`${what} is not an allowed host for this document`);
    } else if (!privateNetwork && privateHost(url.hostname)) {
        throw new Error(`${what} is on this machine or a private network, which this document may not reach (the host can allow it with allowPrivateNetwork(), or list it in allowHttp([...]))`);
    }
}

/**
 * Http.fetch for JavaScript hosts (added by StofDoc.allowHttp()).
 * Same signature and response map as the native runtime's Http.fetch, so documents behave the same everywhere:
 * Http.fetch(url, method = 'get', body = null, headers = null, timeout (seconds) = null, query = null, bearer = null)
 * -> map { status, ok, headers, content_type, bytes }
 */
async function stofFetch(
    allowed: Set<string> | null,
    privateNetwork: () => boolean,
    url: string,
    method: string = 'get',
    body: string | Uint8Array | null = null,
    headers: StofHeaders = null,
    timeout: number | null = null,
    query: StofHeaders = null,
    bearer: string | null = null,
): Promise<Map<string, unknown>> {
    const entries = (values: StofHeaders): [string, string][] => {
        if (!values) return [];
        const pairs = values instanceof Map ? Array.from(values.entries()) : Object.entries(values);
        return pairs.map(([key, value]) => [String(key), String(value)]);
    };
    // @ts-ignore - location only exists in browsers (resolves relative URLs there)
    const target = new URL(url, typeof location !== 'undefined' ? location.href : undefined);
    for (const [key, value] of entries(query)) target.searchParams.append(key, value);
    const privateOk = privateNetwork();
    checkHost(target, allowed, privateOk, false);

    const requestHeaders = new Headers(entries(headers));
    if (bearer) requestHeaders.set('Authorization', `Bearer ${bearer}`);

    // On servers (no window), follow redirects here so every hop is checked before it's sent. Browsers keep their
    // own redirects (a manual redirect is opaque there) and the final URL is checked.
    // @ts-ignore - window only exists in browsers
    const manual = typeof window === 'undefined';
    let init: RequestInit = {
        method: (method || 'get').toUpperCase(),
        headers: requestHeaders,
        body: (body ?? undefined) as BodyInit | undefined,
        signal: timeout ? AbortSignal.timeout(timeout * 1000) : undefined,
        redirect: manual ? 'manual' : 'follow',
    };
    let current = target;
    let response: Response;
    for (let hops = 0; ; hops++) {
        response = await fetch(current, init);
        const location = response.headers.get('location');
        if (!manual || response.status < 300 || response.status > 399 || !location) break;
        if (hops >= 10) throw new Error('too many redirects');
        const next = new URL(location, current);
        checkHost(next, allowed, privateOk, true);
        await response.body?.cancel();
        const method = init.method ?? 'GET';
        if (response.status === 303 || ((response.status === 301 || response.status === 302) && method === 'POST')) {
            init = { ...init, method: 'GET', body: undefined }; // as fetch does
        }
        if (next.origin !== current.origin) { // as fetch does: credentials don't follow to another origin
            const forwarded = new Headers(init.headers);
            forwarded.delete('authorization');
            forwarded.delete('cookie');
            init = { ...init, headers: forwarded };
        }
        current = next;
    }
    if (!manual && response.url) checkHost(new URL(response.url), allowed, privateOk, true);

    const responseHeaders = new Map<string, string>();
    response.headers.forEach((value, key) => responseHeaders.set(key, value));
    const result = new Map<string, unknown>();
    result.set('status', response.status);
    result.set('ok', response.ok);
    result.set('headers', responseHeaders);
    result.set('content_type', response.headers.get('content-type') ?? 'text/plain');
    result.set('bytes', new Uint8Array(await response.arrayBuffer()));
    return result;
}


/**
 * A field or function in a Stof text (see `StofDoc.outline`).
 */
export interface StofOutlineDef {
    kind: 'field' | 'fn';
    /** Path from the top level, ending with the name. */
    path: string[];
    name: string;
    /** A field whose value is an object written inline (it can have children). */
    object: boolean;
    /** Where the name is (1-based line, byte column). */
    line: number;
    col: number;
    /** Its source, as if written at column 0: a field's value, or a function from `fn` to the end of its body. */
    value: string;
}

/**
 * An edit to a Stof text (see `StofDoc.edit`). A path is a list of names, or names separated by dots.
 */
export type StofEdit =
    | { set: string | string[]; value: unknown }
    | { set: string | string[]; stof: string }
    | { set_fn: string | string[]; stof: string }
    | { remove: string | string[] }
    | { rename: string | string[]; to: string };


/**
 * Stof document.
 */
export class StofDoc {
    static readonly VERSION = '0.10.11';
    private static initialized?: Promise<unknown>;
    private static wasmMutex = new WasmMutex();
    stof: Stof;
    /** Why the engine trapped in this document (it can't be used again), or null. */
    crashed: string | null = null;
    private privateNetwork = false;


    /**
     * Initialize Stof WASM.
     * Use initStof() instead for automatic environment detection.
     */
    static initialize(data?: BufferSource | string): Promise<unknown> {
        if (StofDoc.initialized) {
            return StofDoc.initialized;
        }
        if (data) {
            StofDoc.initialized = init({ module_or_path: data });
        } else {
            StofDoc.initialized = init();
        }
        return StofDoc.initialized;
    }


    /**
     * The fields and functions in a Stof text, with their paths, in the order they're written.
     * Throws if the text doesn't parse. Types the text uses don't need to be defined.
     * Note: make sure initialize has been called on the wasm.
     */
    static outline(text: string): StofOutlineDef[] {
        return JSON.parse(stofOutline(text)) as StofOutlineDef[];
    }


    /**
     * Edit a Stof text in place: set, add, remove, or rename fields and functions by path. Everything else in the
     * text (comments, formatting, other definitions) stays exactly as it was. Throws (changing nothing) if the
     * text or the result doesn't parse, or an edit doesn't make sense.
     * Values are JSON values, or Stof source with `stof` (Ex. `{ set: 'limit.value', stof: '1GB' }`).
     * Note: make sure initialize has been called on the wasm.
     */
    static edit(text: string, edits: StofEdit[]): string {
        return stofEdit(text, JSON.stringify(edits));
    }


    /**
     * Constructor.
     * Make sure to call initalize before using.
     */
    constructor(stof: Stof = new Stof()) {
        this.stof = guardStof(stof, this);
    }


    /**
     * Run a task to the end, one step at a time: each step holds the call gate (one document runs in the engine at
     * a time), and every 20 ms a step yields to the event loop so host promises and timers run. A library function
     * can call back into this document with the async methods (await doc.call(..)): those run between steps.
     * Each step is a synchronous call into the engine, so a trap fails this call (guardStof) instead of escaping.
     */
    private async drive<T>(task: StofTask, finish: (task: StofTask) => T): Promise<T> {
        try {
            for (;;) {
                try {
                    await StofDoc.wasmMutex.acquire();
                } catch (error) {
                    this.stof.fail(task, `the host's call gate failed to acquire: ${error instanceof Error ? error.message : String(error)}`);
                    break;
                }
                let status: number;
                try {
                    status = this.stof.step(task);
                } finally {
                    StofDoc.wasmMutex.release();
                }
                if (status === 0) break;
                if (status === 2) await yieldToEventLoop();
            }
            return finish(task);
        } finally {
            try { task.free(); } catch { /* already gone with a crashed engine */ }
        }
    }


    /**
     * Ensure stof is initialized before creating a new instance.
     */
    static async new(src?: string | Record<string, unknown> | Uint8Array, format: string = "stof", profile: 'prod' | 'test' = 'prod'): Promise<StofDoc> {
        await initStof();
        const doc = new StofDoc();
        if (src) doc.parse(src, format, undefined, profile);
        return doc;
    }


    /**
     * Parse a JS object into a StofDoc.
     */
    static async parse(obj: Record<string, unknown>): Promise<StofDoc> {
        const doc = await StofDoc.new();
        doc.stof.objImport(obj, null);
        return doc;
    }


    /**
     * Sync parse a JS object into a StofDoc.
     * Note: make sure initialize has been called on the wasm.
     */
    static sync_parse(obj: Record<string, unknown>): StofDoc {
        const doc = new StofDoc();
        doc.stof.objImport(obj, null);
        return doc;
    }


    /**
     * Parse string source, array, or a JS record.
     */
    parse(src: string | Record<string, unknown> | Uint8Array, format: string = "stof", node: string | null = null, profile: 'prod' | 'test' = 'prod'): boolean {
        if (typeof src === 'string') {
            return this.stof.stringImport(src, format, node, profile);
        } else if (src instanceof Uint8Array) {
            return this.stof.binaryImport(src, format, node, profile);
        }
        return this.stof.objImport(src, node);
    }


    /**
     * Add JS library function.
     * The function can call back into this document with the async methods (await doc.call(..), doc.callSource(..)),
     * which run once Stof's current step is done. Synchronous methods (get, set, sync_call, stringify, ..) called
     * before the function's first await find the document busy and throw; call them after an await instead.
     */
    // deno-lint-ignore ban-types
    lib(library: string, name: string, func: Function, is_async?: boolean, params?: string[]) {
        const docid = this.stof.docid();
        const async_fn = is_async ?? func.constructor.name === "AsyncFunction";
        const stofFunc = new StofFunc(docid, library, name, func, async_fn);
        if (params) stofFunc.setParams(params); // lets Stof call it with named arguments
        this.stof.js_library_function(stofFunc);
    }


    /**
     * Give this document network access: the Http library, with Http.fetch backed by the JavaScript fetch API.
     * Off by default: a document can only read and change itself and call the functions you add with lib().
     * With hosts, requests (and redirects) to any other host fail. An entry is a host name (any port,
     * Ex. "api.example.com") or a host and port ("localhost:8080").
     * Without hosts, any public host: this machine and private networks (localhost, 10.x, 192.168.x, 169.254.x where
     * cloud metadata services live, `*.internal`, ...) stay off limits unless you call allowPrivateNetwork().
     * Addresses and names are checked as written (and on every redirect, on servers); JavaScript can't see what a
     * name resolves to, so list the hosts when that matters.
     */
    allowHttp(hosts?: string[]): void {
        this.stof.allowHttp(); // Http helpers (Http.text, Http.parse, Http.success, ...)
        const allowed = hosts ? new Set(hosts.map((host) => host.trim().toLowerCase())) : null;
        const privateNetwork = () => this.privateNetwork;
        const hostFetch = (...args: unknown[]) => (stofFetch as Function)(allowed, privateNetwork, ...args);
        this.lib('Http', 'fetch', hostFetch, true, ['url', 'method', 'body', 'headers', 'timeout', 'query', 'bearer']);
    }


    /**
     * Let a document allowed any host (allowHttp() without a list) reach this machine and private networks too.
     * Off by default. Prefer listing the hosts it needs (allowHttp(['localhost:8080'])).
     */
    allowPrivateNetwork(): void {
        this.privateNetwork = true;
    }


    /**
     * How long a call can run, in milliseconds, before it fails with an execution timeout (null for no limit).
     * Defaults to 120,000 (2 minutes).
     */
    setMaxExecutionTime(ms: number | null): void {
        this.stof.setMaxExecutionTime(ms ?? undefined);
    }


    /**
     * How much memory a call can use, in bytes, before it fails with a memory limit error (null for no limit).
     * Defaults to 268,435,456 (256 MB). A document that runs away (Ex. a string doubled in a loop) fails with an
     * error instead of running the engine out of memory, which can't be recovered from.
     */
    setMaxMemory(bytes: number | null): void {
        this.stof.setMaxMemory(bytes ?? undefined);
    }


    /**
     * Elements the html format may write (Ex. the components your page renders): only these, instead of the
     * standard HTML elements and custom elements. Documents can't change it. Calling it again adds elements.
     */
    allowHtmlTags(tags: string[]): void {
        this.stof.allowHtmlTags(tags);
    }


    /**
     * Seal an object (a path, or a type as "<Name>") and everything under it: its functions can be called but not
     * changed, by this document's code or by you (until unseal). Fields still change unless `fields` is true.
     * With `digest`, only if the object's digest is that; with `publicKey` and `signature`, only if its digest is
     * signed by that key (Ed25519, base64 or hex). Throws (sealing nothing) otherwise. Returns the digest.
     * Ex. doc.seal('<Limitr>', { publicKey: LIMITR_KEY, signature: ENGINE_SIGNATURE })
     */
    seal(path: string, options: { fields?: boolean, digest?: string, publicKey?: string, signature?: string } = {}): string {
        return this.stof.seal(path, options.fields ?? false, options.digest, options.publicKey, options.signature);
    }


    /**
     * Unseal an object that seal() sealed.
     */
    unseal(path: string): void {
        this.stof.unseal(path);
    }


    /**
     * The seal digest of an object (a path, or a type as "<Name>"): a SHA-256 (hex) over its functions and
     * structure, and its fields' values when `fields` is true. The same for the same document, parsed or loaded.
     */
    digest(path: string, fields: boolean = false): string {
        return this.stof.digest(path, fields);
    }


    /**
     * Sign a seal digest with an Ed25519 secret key (base64 or hex, 32 bytes, Ex. `openssl rand -base64 32`).
     * Returns the signature (base64). Sign at build time; ship the public key and signature with the document.
     */
    static sign(digest: string, secretKey: string): string {
        return stofSign(digest, secretKey);
    }


    /**
     * Is this an Ed25519 signature over this seal digest by this public key?
     */
    static verify(digest: string, publicKey: string, signature: string): boolean {
        return stofVerify(digest, publicKey, signature);
    }


    /**
     * The Ed25519 public key (base64) for a secret key.
     */
    static publicKey(secretKey: string): string {
        return stofPublicKey(secretKey);
    }


    /**
     * Attributes the html format writes only when you allow them: `style`, `action`, and `formaction` (off by
     * default, since CSS can restyle or cover your page and forms can post elsewhere). Documents can't change it.
     * Calling it again adds attributes.
     */
    allowHtmlAttributes(attributes: string[]): void {
        this.stof.allowHtmlAttributes(attributes);
    }


    /**
     * Keep each function's source text when parsing from now on (off by default), so `stringify('stof')` writes
     * functions as Stof instead of data blocks. It costs the memory of the text. Call it before parsing.
     */
    keepSource(keep: boolean = true): void {
        this.stof.keepSource(keep);
    }


    /**
     * Print nothing to the console (off by default): no parse warnings, test progress, or pln/err/dbg output.
     * Results and errors still come back (Ex. `test` still returns its report).
     */
    quiet(quiet: boolean = true): void {
        this.stof.quiet(quiet);
    }


    /**
     * Run this document with a given set of Stof attributes.
     * Will run all #[main] functions by default.
     */
    async run(attr: string | string[] = 'main'): Promise<string> {
        await Promise.resolve(); // start after the current step, like before (a library function can call back)
        const task = this.stof.start_run(attr);
        return await this.drive(task, (task) => this.stof.report(task));
    }


    /**
     * Run this document with a given set of Stof attributes (synchronously).
     * Will run all #[main] functions by default.
     * Note: any async TS library functions called will not work with synchronous exec (ex. fetch).
     */
    sync_run(attr: string | string[] = 'main'): string {
        return this.stof.sync_run(attr);
    }


    /**
     * Call a specific Stof function by path/name.
     */
    async call(path: string, ...args: unknown[]): Promise<unknown> {
        if (!path.includes('.')) path = 'root.' + path; // assume root node if not specified
        await Promise.resolve(); // start after the current step, like before (a library function can call back)
        const task = this.stof.start_call(path, args);
        return await this.drive(task, (task) => this.stof.result(task));
    }


    /**
     * Call a specific Stof function by path/name.
     * Note: any async TS library functions called will not work with synchronous exec (ex. fetch).
     */
    sync_call(path: string, ...args: unknown[]): unknown {
        if (!path.includes('.')) path = 'root.' + path; // assume root node if not specified
        return this.stof.sync_call(path, args);
    }


    /**
     * Call a function from Stof source without adding it to this document (Ex. a one-off script or check).
     * The source's first function runs as if it were defined on node (an object's path or ID, the root by default),
     * and everything the source declares is removed after the call.
     * Ex. await doc.callSource('fn total(qty: int) -> float { self.price * qty }', [3]);
     */
    async callSource(src: string, args: unknown[] = [], node: string | null = null): Promise<unknown> {
        await Promise.resolve(); // start after the current step, like before (a library function can call back)
        const task = this.stof.start_call_source(src, args, node);
        return await this.drive(task, (task) => this.stof.result(task));
    }


    /**
     * Call a function from Stof source without adding it to this document (synchronously).
     * Note: any async TS library functions called will not work with synchronous exec (ex. fetch).
     */
    sync_callSource(src: string, args: unknown[] = [], node: string | null = null): unknown {
        return this.stof.sync_call_source(src, args, node);
    }


    /**
     * Remove a field, function, or object by path (Ex. "config.staging"), like drop() in Stof.
     * A field that holds an object drops the object too, unless shallow (then only the field is removed).
     * Returns true if something was removed.
     */
    remove(path: string, shallow: boolean = false): boolean {
        return this.stof.remove(path, shallow);
    }


    /**
     * Get a value from this graph by path and an optional starting object ID.
     */
    get(path: string, start_obj_id: string | null = null): unknown {
        if (!path.includes('.')) path = 'self.' + path;
        return this.stof.get(path, start_obj_id);
    }


    /**
     * Set a value on this graph by path.
     * Returns true if successfully set.
     */
    set(path: string, value: unknown, start_obj_id: string | null = null): boolean {
        if (!path.includes('.')) path = 'self.' + path;
        return this.stof.set(path, value, start_obj_id);
    }


    /**
     * Stringify this doc into a format (JSON by default).
     * Node is an object's path (Ex. "config.production") or ID; the whole document by default.
     * Computed values (Ex. #[attr] functions in html) are called first; for one that calls an async host function,
     * stringify from Stof instead (Ex. callSource('fn f() -> str { stringify("html", self) }', [], node)).
     */
    stringify(format: string = "json", node: string | null = null): string {
        return this.stof.stringExport(format, node);
    }


    /**
     * Blobify this doc (or a specific node) into a format (JSON by default).
     * Node is an object's path (Ex. "config.production") or ID; the whole document by default.
     */
    blobify(format: string = "json", node: string | null = null): Uint8Array {
        return this.stof.binaryExport(format, node);
    }


    /**
     * To JS record.
     * Node is an object's path (Ex. "config.production") or ID; the whole document by default.
     */
    record(node: string | null = null): Record<string, unknown> {
        return JSON.parse(this.stringify('json', node));
    }


    /**
     * Free WASM resources.
     * Call this when you're done with the document to prevent memory leaks.
     */
    dispose(): void {
        if (this.stof && typeof this.stof.free === 'function') {
            this.stof.free();
        }
    }


    /**
     * Create a deep copy of this document.
     */
    async clone(): Promise<StofDoc> {
        const serialized = this.stof.binaryExport('bstf', null);
        const doc = await StofDoc.new();
        doc.stof.binaryImport(serialized, 'bstf', null, 'prod');
        return doc;
    }
}
