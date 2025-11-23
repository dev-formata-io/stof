//
// Copyright 2024 Formata, Inc. All rights reserved.
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
     * Parse Stof source.
     */
    parse(stof: string, node: string | null = null): boolean {
        return this.stof.parse(stof, node);
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
     */
    async run(attr: string | string[] = 'main'): Promise<string> {
        return await this.stof.run(attr);
    }


    /**
     * Call a specific Stof function by path.
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
