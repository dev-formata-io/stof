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
 * Stof document.
 */
export class Doc {
    /** Initialized? */
    private static initialized?: Promise<void>;

    /** Stof Document. */
    stof: Stof;


    /**
     * Initialize Stof WASM.
     */
    static async initialize(data?: unknown): Promise<void> {
        // @ts-ignore this exists
        return Doc.initialized ?? await (Doc.initialized = init(data));
    }


    /**
     * Constructor.
     * Make sure to call initalize before using.
     */
    constructor(stof: Stof) {
        this.stof = stof;
    }


    /**
     * Create & initialize (if needed).
     */
    static async new(): Promise<Doc> {
        await Doc.initialize();
        return new Doc(new Stof());
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
    lib(library: string, name: string, func: Function) {
        this.stof.js_library_function(new StofFunc(library, name, func));
    }


    /**
     * Run this document with a given set of attributes.
     */
    run(attr: string | string[] = 'main'): string {
        return this.stof.run(attr);
    }
}
