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
import init, { Stof, StofFunc } from './pkg/stof.js';
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
 * Http.fetch for JavaScript hosts (added by StofDoc.allowHttp()).
 * Same signature and response map as the native runtime's Http.fetch, so documents behave the same everywhere:
 * Http.fetch(url, method = 'get', body = null, headers = null, timeout (seconds) = null, query = null, bearer = null)
 * -> map { status, ok, headers, content_type, bytes }
 */
async function stofFetch(
    allowed: Set<string> | null,
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
    if (!hostAllowed(target, allowed)) throw new Error(`${target.origin} is not an allowed host for this document`);

    const requestHeaders = new Headers(entries(headers));
    if (bearer) requestHeaders.set('Authorization', `Bearer ${bearer}`);

    const response = await fetch(target, {
        method: (method || 'get').toUpperCase(),
        headers: requestHeaders,
        body: (body ?? undefined) as BodyInit | undefined,
        signal: timeout ? AbortSignal.timeout(timeout * 1000) : undefined,
    });
    if (response.url && !hostAllowed(new URL(response.url), allowed)) {
        throw new Error(`redirected to ${response.url}, which is not an allowed host`);
    }

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
 * Stof document.
 */
export class StofDoc {
    static readonly VERSION = '0.10.4';
    private static initialized?: Promise<unknown>;
    private static wasmMutex = new WasmMutex();
    stof: Stof;


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
     * Constructor.
     * Make sure to call initalize before using.
     */
    constructor(stof: Stof = new Stof()) {
        this.stof = stof;
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
     */
    allowHttp(hosts?: string[]): void {
        this.stof.allowHttp(); // Http helpers (Http.text, Http.parse, Http.success, ...)
        const allowed = hosts ? new Set(hosts.map((host) => host.trim().toLowerCase())) : null;
        const hostFetch = (...args: unknown[]) => (stofFetch as Function)(allowed, ...args);
        this.lib('Http', 'fetch', hostFetch, true, ['url', 'method', 'body', 'headers', 'timeout', 'query', 'bearer']);
    }


    /**
     * How long a call can run, in milliseconds, before it fails with an execution timeout (null for no limit).
     * Defaults to 120,000 (2 minutes).
     */
    setMaxExecutionTime(ms: number | null): void {
        this.stof.setMaxExecutionTime(ms ?? undefined);
    }


    /**
     * Run this document with a given set of Stof attributes.
     * Will run all #[main] functions by default.
     */
    async run(attr: string | string[] = 'main'): Promise<string> {
        const acquire = () => StofDoc.wasmMutex.acquire();
        const release = () => StofDoc.wasmMutex.release();
        return await this.stof.run_with_gate(attr, acquire, release);
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
        const acquire = () => StofDoc.wasmMutex.acquire();
        const release = () => StofDoc.wasmMutex.release();
        return await this.stof.call_with_gate(path, args, acquire, release);
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
     */
    stringify(format: string = "json", node: string | null = null): string {
        return this.stof.stringExport(format, node);
    }


    /**
     * Blobify this doc (or a specific node) into a format (JSON by default).
     */
    blobify(format: string = "json", node: string | null = null): Uint8Array {
        return this.stof.binaryExport(format, node);
    }


    /**
     * To JS record.
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
