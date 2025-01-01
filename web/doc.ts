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

import init, { type StofData, StofDoc, type StofField, type StofFunc, type StofNode } from './pkg/stof.js';


/**
 * Stof Document.
 */
export class Stof {
    /** Initialized? */
    private static initialized?: Promise<void>;

    /** Stof Document. */
    doc: StofDoc;


    /**
     * Initailize Stof.
     * Make sure to call this before using any Stof WebAssembly.
     */
    static async initialize(): Promise<void> {
        // @ts-ignore this exists
        return Stof.initialized ?? await (Stof.initialized = init());
    }


    /**
     * Constructor.
     */
    constructor(doc: StofDoc) {
        this.doc = doc;
        this.insertConsole();
    }


    /**
     * Create a new document.
     * Calls and waits for 'initialize' if wasm is not already loaded.
     */
    static async create(name: string = 'doc', record: Record<string, unknown> = {}): Promise<Stof> {
        await Stof.initialize();
        return new Stof(StofDoc.js(name, record));
    }


    /**
     * New document.
     * Must call 'initialize' before using this.
     */
    static doc(name: string = 'doc', record: Record<string, unknown> = {}): Stof {
        return new Stof(StofDoc.js(name, record));
    }


    /**
     * Name of this document.
     */
    get name(): string {
        return this.doc.name();
    }


    /**
     * ID of this document.
     */
    get id(): string {
        return this.doc.id();
    }


    /*****************************************************************************
     * Formats.
     *****************************************************************************/
    
    /**
     * Available formats.
     */
    get availableFormats(): string[] {
        return this.doc.availableFormats();
    }


    /**
     * Format content type.
     */
    formatContentType(format: string): string | undefined {
        return this.doc.formatContentType(format);
    }


    /**
     * Import bytes.
     * Will throw an error if the format doesn't exist or a problem occurs.
     */
    importBytes(format: string, contentType: string, bytes: Uint8Array, as_name = ''): boolean {
        return this.doc.headerImport(format, contentType, bytes, as_name);
    }


    /**
     * Import string.
     * Will throw an error if the format doesn't exist of a problem occurs.
     */
    importString(format: string, src: string, as_name = ''): boolean {
        return this.doc.stringImport(format, src, as_name);
    }


    /**
     * Export bytes.
     */
    exportBytes(format: string, node?: StofNode): Uint8Array {
        if (node) {
            return this.doc.exportBytesFor(format, node);
        }
        return this.doc.exportBytes(format);
    }


    /**
     * Export string.
     */
    exportString(format: string, node?: StofNode): string {
        if (node) {
            return this.doc.exportStringFor(format, node);
        }
        return this.doc.exportString(format);
    }


    /*****************************************************************************
     * Libraries.
     *****************************************************************************/
    
    /**
     * Available libraries.
     */
    get availableLibraries(): string[] {
        return this.doc.availableLibraries();
    }


    /**
     * Insert library functions.
     *
     * ```
     * doc.insertLibrary('console', [
     *  ['log', (...params: unknown[])=>console.log(...params)]
     * ]);
     * 
     * // Can now use 'console.log' function in Stof.
     * ```
     */
    insertLibrary(lib: string, functions: [string, unknown][]) {
        for (const [name, func] of functions) {
            this.doc.insertLibFunc(lib, name, func);
        }
        this.doc.createLibs();
    }


    /**
     * Insert console library.
     */
    insertConsole() {
        this.insertLibrary('console', [
            ['log', (...params: unknown[])=>console.log(...params)],
            ['trace', (...params: unknown[])=>console.trace(...params)],
            ['error', (...params: unknown[])=>console.error(...params)],
            ['info', (...params: unknown[])=>console.info(...params)],
            ['warn', (...params: unknown[])=>console.warn(...params)],
        ]);
    }


    /*****************************************************************************
     * Stof Data (fields and functions).
     *****************************************************************************/
    
    /**
     * Get a value by path (dot separated).
     * If the path points to a field, the value will be retrieved.
     * If the path points to a function, it will be called. Param is 'get' if any.
     */
    get(path: string): unknown {
        return this.doc.get(path);
    }


    /**
     * Get a field by path (dot separated) with an optional start.
     * Without a starting node, Stof will look for a root with the first name in the path.
     */
    field(path: string, node?: StofNode): StofField | undefined {
        if (node) {
            return this.doc.field(path, node);
        }
        return this.doc.fieldFromRoot(path);
    }


    /**
     * Get a function by path (dot separated) with an optional start.
     * Without a starting node, Stof will look for a root with the first name in the path.
     */
    func(path: string, node?: StofNode): StofFunc | undefined {
        if (node) {
            return this.doc.func(path, node);
        }
        return this.doc.funcFromRoot(path);
    }


    /**
     * Call a function by path (dot separated) from a root.
     * Quick alternative to calling "func" and then "call" on that function.
     * Will throw an error if the function does not exist or does not execute properly.
     */
    call(path: string, params: unknown[]): unknown {
        return this.doc.callFunc(path, params);
    }


    /**
     * Run this document, calling all #[main] functions.
     *
     * @param node A context node, where only #[main] functions on or under this node will be called.
     */
    run(node?: StofNode) {
        if (node) {
            this.doc.runAt(node);
        } else {
            this.doc.run();
        }
    }


    /*****************************************************************************
     * Graph.
     *****************************************************************************/
    
    /**
     * Main root.
     */
    get mainRoot(): StofNode | undefined {
        return this.doc.mainRoot();
    }


    /**
     * Root by name.
     */
    rootByName(name: string): StofNode | undefined {
        return this.doc.rootByName(name);
    }


    /**
     * Is a root?
     */
    isRoot(node: StofNode): boolean {
        return this.doc.isRoot(node);
    }


    /**
     * Get all roots.
     */
    roots(): StofNode[] {
        return this.doc.roots();
    }


    /**
     * Insert a node.
     */
    insertNode(name: string, parent?: StofNode): StofNode {
        if (parent) {
            return this.doc.insertNode(name, parent);
        }
        return this.doc.insertRoot(name);
    }


    /**
     * Remove a node.
     * Deletes all data on the node from the graph is this node is the only reference.
     */
    removeNode(node: StofNode): boolean {
        return this.doc.removeNode(node);
    }


    /**
     * Put data onto a node.
     */
    putData(node: StofNode, data: StofData): boolean {
        return this.doc.putData(node, data);
    }


    /**
     * Remove data.
     *
     * @param node Only remove the data from this node.
     */
    removeData(data: StofData, node?: StofNode): boolean {
        if (node) {
            return this.doc.removeDataFrom(data, node);
        }
        return this.doc.removeData(data);
    }


    /*****************************************************************************
     * Custom, External Data.
     *****************************************************************************/
    
    /**
     * Create data helper.
     * Need to pass a node and a val to create data with nanoid ID.
     * Pass an ID to give the data a specific ID - will overwrite data if already present.
     * Data will be marked as dirty val when created.
     *
     * @param node Node Ref.
     * @param val Data value.
     * @param id Optional ID.
     */
    createData(node?: StofNode, val?: unknown, id?: string): StofData | undefined {
        if (node === undefined) node = this.doc.mainRoot();
        if (node && val && id) {
            return this.doc.createDataWithId(node, id, val);
        } else if (node && val) {
            return this.doc.createData(node, val);
        }
        return undefined;
    }


    /**
     * Stof data helper for getting some data from a ref.
     *
     * @param ref Data Ref to get value from.
     * @param object Whether the data is an object or not.
     * @returns The data value pointed to by 'ref' if it exists.
     */
    getValue<T = { [key: string]: unknown }>(ref?: StofData, object = true): T | undefined {
        if (object && ref) {
            const val = ref.getValue(this.doc);
            if (val) {
                const map = val as Map<string, unknown>;
                return removeMaps(map) as T | undefined;
            }
        } else if (ref) {
            return ref.getValue(this.doc) as T | undefined;
        }
        return undefined;
    }


    /**
     * Stof data helper for setting a value to a data ref.
     *
     * @param ref Data ref to set value to (if it exists).
     * @param val The value to set on the data.
     */
    setValue(ref?: StofData, val?: unknown): boolean {
        if (val !== undefined && ref && ref.exists(this.doc)) {
            if (typeof val === 'object') {
                ref.setValue(this.doc, removeObjects(val as Record<string, unknown>));
            } else {
                ref.setValue(this.doc, val);
            }
            return true;
        }
        return false;
    }


    /*****************************************************************************
     * Deadpool & Dirty.
     *****************************************************************************/
    
    /**
     * Flush node deadpool.
     * These are all of the nodes that were removed from the graph.
     */
    flushNodeDeadpool(): Record<string, unknown>[] {
        return this.doc.flush_node_deadpool();
    }


    /**
     * Flush data deadpool.
     * These are all of the data that has been removed from the graph.
     */
    flushDataDeadpool(): Record<string, unknown>[] {
        return this.doc.flush_data_deadpool();
    }


    /**
     * Flush nodes.
     * These are nodes that have been marked as 'dirty' with some symbol.
     */
    flushNodes(limit = -1): StofNode[] {
        return this.doc.flush_nodes(limit);
    }


    /**
     * Flush data.
     * This is data that has been marked as 'dirty' with some symbol.
     */
    flushData(limit = -1): StofData[] {
        return this.doc.flush_data(limit);
    }
}

/**
 * Remove all maps from save data.
 */
export function removeMaps(data: Map<string, unknown>): Record<string, unknown> {
    const names: string[] = [];
    for (const [name, val] of data) {
        if (val instanceof Map) {
            names.push(name);
        } else if (val instanceof Array) {
            removeArrayMaps(val);
        }
    }
    for (const name of names) {
        const val = data.get(name) as Map<string, unknown>;
        removeMaps(val);
        data.set(name, Object.fromEntries(val));
    }
    return Object.fromEntries(data);
}


/**
 * Remove all maps from array save data.
 */
function removeArrayMaps(array: Array<unknown>) {
    const indices: number[] = [];
    for (let i = 0; i < array.length; i++) {
        if (array[i] instanceof Map) {
            indices.push(i);
        } else if (array[i] instanceof Array) {
            removeArrayMaps(array[i] as Array<unknown>);
        }
    }
    for (const index of indices) {
        const mp = array[index] as Map<string, unknown>;
        array[index] = removeMaps(mp);
    }
}


/**
 * Remove objects.
 */
export function removeObjects(obj: Record<string, unknown>): Map<string, unknown> {
    const data = new Map(Object.entries(obj));
    const names: string[] = [];
    for (const [name, val] of data) {
        if (val instanceof Array) {
            removeArrayObjects(val);
        } else if (typeof val === 'object') {
            names.push(name);
        }
    }
    for (const name of names) {
        const val = data.get(name) as Record<string, unknown>;
        data.set(name, removeObjects(val));
    }
    return data;
}


/**
 * Remove all objects from array load data.
 */
function removeArrayObjects(array: Array<unknown>) {
    const indices: number[] = [];
    for (let i = 0; i < array.length; i++) {
        if (array[i] instanceof Array) {
            removeArrayObjects(array[i] as Array<unknown>);
        } else if (typeof array[i] === 'object') {
            indices.push(i);
        }
    }
    for (const index of indices) {
        const mp = array[index] as Record<string, unknown>;
        array[index] = removeObjects(mp);
    }
}


/**
 * Remove all undefined values from an object.
 */
export function removeUndefined(json: Record<string, unknown>) {
    Object.keys(json).forEach(key => {
        if (json[key] === undefined) {
            delete json[key];
        } else if (typeof json[key] === 'object') {
            removeUndefined(json[key] as Record<string, unknown>);
        }
    });
}
