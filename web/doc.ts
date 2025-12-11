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
 * Template function for a document.
 * Stof must be initialized before with `await Doc.initialize()`.
 */
export function stof(strings: TemplateStringsArray, ...values: unknown[]): StofDoc {
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
 * Stof document.
 */
export class StofDoc {
    /** Initialized? */
    private static initialized?: Promise<void>;

    /** Stof Document. */
    stof: Stof;


    /**
     * Initialize Stof WASM.
     */
    static async initialize(data?: unknown): Promise<void> {
        // @ts-ignore this exists
        return StofDoc.initialized ?? await (StofDoc.initialized = init(data));
    }


    /**
     * Constructor.
     * Make sure to call initalize before using.
     */
    constructor(stof: Stof = new Stof()) {
        this.stof = stof;
    }


    /**
     * Create & initialize (if needed).
     */
    static async new(): Promise<StofDoc> {
        await StofDoc.initialize();
        return new StofDoc();
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
     * Parse string source or a JS record.
     */
    parse(src: string | Record<string, unknown>, format: string = "stof", node: string | null = null, profile: 'prod' | 'test' = 'prod'): boolean {
        if (typeof src === 'string') {
            return this.stof.stringImport(src, format, node, profile);
        }
        return this.stof.objImport(src, node);
    }


    /**
     * Add JS library function.
     */
    // deno-lint-ignore ban-types
    lib(library: string, name: string, func: Function, is_async: boolean = false) {
        this.stof.js_library_function(new StofFunc(library, name, func, is_async));
    }


    /**
     * Run this document with a given set of Stof attributes.
     * Will run all #[main] functions by default.
     */
    async run(attr: string | string[] = 'main'): Promise<string> {
        return await this.stof.run(attr);
    }


    /**
     * Call a specific Stof function by path/name.
     */
    async call(path: string, ...args: unknown[]): Promise<unknown> {
        if (!path.includes('.')) path = 'root.' + path; // assume root node if not specified
        return await this.stof.call(path, args);
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
     * To JS record.
     */
    record(node: string | null = null): Record<string, unknown> {
        return JSON.parse(this.stringify('json', node));
    }


    /*****************************************************************************
     * Network.
     *****************************************************************************/

    /**
     * Send Stof string body as an HTTP request.
     */
    static async send(url: string, stof: string, method: string = 'POST', bearer?: string, headers: Record<string, string> = {} as Record<string, string>): Promise<Response> {
        headers['Content-Type'] = 'application/stof';
        if (bearer !== undefined) headers['Authorization'] = `Bearer ${bearer}`;
        return await fetch(url, {
            method,
            headers: headers as HeadersInit,
            body: stof
        });
    }
}
