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

// @ts-ignore wasm exists...
//import stofWasm from '../pkg/stof_bg.wasm';
import init, { StofData, StofDoc, StofNode } from '../pkg/stof.js';


/**
 * STOF interface.
 */
export class STOF {
    /** Initialized? */
    private static initialized?: Promise<void>;


    /**
     * Initailize Stof.
     * Make sure to call this before using anything from Stof.
     */
    static async initialize(): Promise<void> {
        // @ts-ignore this exists
        return STOF.initialized ?? await (STOF.initialized = init());
    }


    /**
     * Stringify a Stof Document.
     */
    static stringify(doc: StofDoc): string {
        return doc.exportString('stof');
    }


    /**
     * Parse Stof string or JS Object into a Stof Document.
     */
    static parse(src: string | Record<string, unknown> | Uint8Array, doc?: StofDoc): StofDoc {
        if (typeof src === 'string') {
            if (doc) {
                doc.stringImport('stof', src, '');
                return doc;
            }
            return new StofDoc('doc', src, 'stof');
        }
        if (doc) {
            if (src instanceof Uint8Array) {
                doc.headerImport('bstof', 'bstof', src, '');
            } else {
                doc.stringImport('json', JSON.stringify(src), '');
            }
            return doc;
        }
        if (src instanceof Uint8Array) {
            return StofDoc.bytes('doc', src, 'bstof');
        }
        return StofDoc.js('doc', src);
    }


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
    static createData(doc: StofDoc, node?: StofNode, val?: unknown, id?: string): StofData | undefined {
        if (node === undefined) node = doc.mainRoot();
        if (node && val && id) {
            return doc.createDataWithId(node, id, val);
        } else if (node && val) {
            return doc.createData(node, val);
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
    static getValue<T = { [key: string]: unknown }>(doc: StofDoc, ref?: StofData, object = true): T | undefined {
        if (object && ref) {
            const val = ref.getValue(doc);
            if (val) {
                const map = val as Map<string, unknown>;
                return removeMaps(map) as T | undefined;
            }
        } else if (ref) {
            return ref.getValue(doc) as T | undefined;
        }
        return undefined;
    }


    /**
     * Stof data helper for setting a value to a data ref.
     *
     * @param ref Data ref to set value to (if it exists).
     * @param val The value to set on the data.
     */
    static setValue(doc: StofDoc, ref?: StofData, val?: unknown): boolean {
        if (val !== undefined && ref && ref.exists(doc)) {
            if (typeof val === 'object') {
                ref.setValue(doc, removeObjects(val as Record<string, unknown>));
            } else {
                ref.setValue(doc, val);
            }
            return true;
        }
        return false;
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
