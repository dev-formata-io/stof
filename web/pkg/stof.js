let wasm;

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_2.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_2.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_export_2.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addToExternrefTable0(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

const StofDataFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stofdata_free(ptr >>> 0, 1));
/**
 * Stof Data.
 */
export class StofData {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StofData.prototype);
        obj.__wbg_ptr = ptr;
        StofDataFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StofDataFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stofdata_free(ptr, 0);
    }
    /**
     * JSON data constructor.
     * Will turn value into serde_json::Value, then create an SData, putting it into the document at 'node'.
     * Only works if node is already in the document.
     * @param {StofDoc} doc
     * @param {StofNode} node
     * @param {any} value
     */
    constructor(doc, node, value) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        const ret = wasm.stofdata_construct(doc.__wbg_ptr, node.__wbg_ptr, value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        StofDataFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Construct a new StofData with an ID and a value.
     * @param {StofDoc} doc
     * @param {StofNode} node
     * @param {string} id
     * @param {any} value
     * @returns {StofData}
     */
    static newWithId(doc, node, id, value) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdata_newWithId(doc.__wbg_ptr, node.__wbg_ptr, ptr0, len0, value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return StofData.__wrap(ret[0]);
    }
    /**
     * Remove this data from every place within the document.
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    remove(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_remove(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Remove this data from a specific node in the document.
     * @param {StofDoc} doc
     * @param {StofNode} node
     * @returns {boolean}
     */
    removeFrom(doc, node) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        const ret = wasm.stofdata_removeFrom(this.__wbg_ptr, doc.__wbg_ptr, node.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * ID constructor.
     * @param {string} id
     * @returns {StofData}
     */
    static new(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdata_new(ptr0, len0);
        return StofData.__wrap(ret);
    }
    /**
     * Get the ID of this reference.
     * @returns {string}
     */
    id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stofdata_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Invalidate this data with a symbol.
     * @param {StofDoc} doc
     * @param {string} symbol
     * @returns {boolean}
     */
    invalidate(doc, symbol) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(symbol, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdata_invalidate(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Invalidate value on this data.
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    invalidateValue(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_invalidateValue(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Dirty?
     * @param {StofDoc} doc
     * @param {string} symbol
     * @returns {boolean}
     */
    dirty(doc, symbol) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(symbol, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdata_dirty(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Any dirty symbols?
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    anyDirty(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_anyDirty(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Validate this data with the symbol.
     * @param {StofDoc} doc
     * @param {string} symbol
     * @returns {boolean}
     */
    validate(doc, symbol) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(symbol, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdata_validate(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Validate value for this data.
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    validate_all(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_validate_all(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Exists?
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    exists(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_exists(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Nodes that contain this data.
     * Data can exist on several nodes at once.
     * @param {StofDoc} doc
     * @returns {(StofNode)[]}
     */
    nodes(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_nodes(this.__wbg_ptr, doc.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Try getting the JSON value of this data.
     * Will only work if the value of this data can be deserialized into serde_json::Value.
     * @param {StofDoc} doc
     * @returns {any}
     */
    getValue(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_getValue(this.__wbg_ptr, doc.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Try setting a JSON value for this data.
     * @param {StofDoc} doc
     * @param {any} value
     * @returns {boolean}
     */
    setValue(doc, value) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_setValue(this.__wbg_ptr, doc.__wbg_ptr, value);
        return ret !== 0;
    }
    /**
     * JSON value of this data as a whole.
     * Can use this to store this value in an external place.
     * @param {StofDoc} doc
     * @returns {any}
     */
    to_json(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_to_json(this.__wbg_ptr, doc.__wbg_ptr);
        return ret;
    }
    /**
     * Loat a JSON representation of a data into a document.
     * Can use this to load data from an external place.
     * @param {StofDoc} doc
     * @param {any} json
     * @returns {boolean}
     */
    static from_json(doc, json) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofdata_from_json(doc.__wbg_ptr, json);
        return ret !== 0;
    }
}

const StofDocFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stofdoc_free(ptr >>> 0, 1));
/**
 * JS Stof Document Interface.
 */
export class StofDoc {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StofDoc.prototype);
        obj.__wbg_ptr = ptr;
        StofDocFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StofDocFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stofdoc_free(ptr, 0);
    }
    /**
     * Construct a new StofDoc with a name.
     * Optionally provide some existing data to load in the format of choice (leave empty if not).
     *
     * If loading a JS object, use 'js' instead, passing the object.
     *
     * Built in formats:
     * - json
     * - stof
     * - toml
     * - xml
     * - yaml
     * - toml
     * - urlencoded
     * @param {string} name
     * @param {string} src
     * @param {string} format
     */
    constructor(name, src, format) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(src, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_new(ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        StofDocFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Construct a new StofDoc using a JS object.
     * This will convert the object into a serde_json::Value before creating a document out of it, capturing it's fields, sub-objects, etc...
     * @param {string} name
     * @param {any} jsobj
     * @returns {StofDoc}
     */
    static js(name, jsobj) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_js(ptr0, len0, jsobj);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return StofDoc.__wrap(ret[0]);
    }
    /**
     * Construct a new StofDoc using bytes and a format.
     * @param {string} name
     * @param {any} bytes
     * @param {string} format
     * @returns {StofDoc}
     */
    static bytes(name, bytes, format) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_bytes(ptr0, len0, bytes, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return StofDoc.__wrap(ret[0]);
    }
    /**
     * Get the ID of this document.
     * This is a unique string ID, generated with nanoid. Can be used for storage, etc...
     * @returns {string}
     */
    id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stofdoc_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get the name of this document.
     * @returns {string}
     */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stofdoc_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Set the name of this document.
     * @param {string} name
     */
    setName(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.stofdoc_setName(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Version of this document.
     * @returns {string}
     */
    version() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stofdoc_version(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get all of the available formats.
     * @returns {(string)[]}
     */
    availableFormats() {
        const ret = wasm.stofdoc_availableFormats(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get the content type for a format.
     * @param {string} format
     * @returns {string | undefined}
     */
    formatContentType(format) {
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_formatContentType(this.__wbg_ptr, ptr0, len0);
        let v2;
        if (ret[0] !== 0) {
            v2 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v2;
    }
    /**
     * Header import.
     * Used for importing bytes (Uint8Arrays) into this document with the given format.
     *
     * If given an explicit 'format' that exists, stof will try to use that one first. Otherwise, stof will look through all of the
     * available formats for one with a content type that matches 'content_type'. If no matches exist, any formats that stof has
     * with a 'format' that is contained in 'content_type' will be used as a fallback. If all of those fail, stof will use its own
     * 'bytes' format to add the raw Vec<u8> as a blob value in the main root under the field name 'bytes'.
     *
     * Ex. utf-8 encoded JSON parsed into the main root: `header_import('json', '', bytes, '')`
     *
     * Ex. utf-8 encoded JSON parsed into the root object named 'Import': `header_import('', 'application/json', bytes, 'Import')`
     *
     * Ex. bstof encoded byte array: `header_import('bstof', 'application/bstof', bytes, '')`
     *
     * Ex. unstructured and unknown format bytes into the path 'Imports.Unknown': `header_import('', '', bytes, 'Imports.Unknown')`.
     * The 'bytes' field with the blob (Vec<u8>) value will exist at `Imports.Unknown.bytes`.
     * @param {string} format
     * @param {string} content_type
     * @param {any} bytes
     * @param {string} as_name
     * @returns {boolean}
     */
    headerImport(format, content_type, bytes, as_name) {
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(content_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(as_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_headerImport(this.__wbg_ptr, ptr0, len0, ptr1, len1, bytes, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * String import.
     * Used for importing/parsing strings into this document with the given format.
     *
     * Ex. JSON string parsed into the main root: `string_import('json', '{ "field": true }', '')`
     *
     * Ex. TOML string parsed into the path 'toml_import': `string_import('toml', toml_src, 'toml_import')`.
     * Now, in this document, all of the toml_src has been put into the location 'root.toml_import'. If toml_src contained a field named
     * 'message' with the value 'hello, world', you can now access it in Stof with 'self.toml_import.message' if in the main root of this doc.
     *
     * Ex. URLencoded string parsed into the root named 'Import': `string_import('urlencoded', 'field=true&another=false', 'Import')`.
     * After this, `assertEq(Import.field, true)` and `assertEq(Import.another, false)`.
     * @param {string} format
     * @param {string} src
     * @param {string} as_name
     * @returns {boolean}
     */
    stringImport(format, src, as_name) {
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(src, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(as_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_stringImport(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * File import.
     * Used for importing/parsing files into this document with the given format.
     *
     * By default, the "fs" (file system) library is not included, so you'll need to implement the following functions yourself:
     * - "fs.read" with one path (str) parameter `doc.insertLibFunc('fs', 'read', (path: string):string => {...}`
     * - "fs.read_blob" with one path (str) parameter `doc.insertLibFunc('fs', 'read_blob', (path: string):Uint8Array => {...}`
     * - "fs.write" with two parameters `doc.insertLibFunc('fs', 'write', (path: string, contents: string) => {...}`
     * - "fs.write_blob" with two parameters `doc.insertLibFunc('fs', 'write_blob', (path: string, contents: Uint8Array) => {...}`
     * @param {string} format
     * @param {string} path
     * @param {string} extension
     * @param {string} as_name
     * @returns {boolean}
     */
    fileImport(format, path, extension, as_name) {
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(extension, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(as_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_fileImport(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Export this document to a string using the format 'format'.
     * @param {string} format
     * @returns {string}
     */
    exportString(format) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.stofdoc_exportString(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Export a node to a string using the format 'format'.
     * @param {string} format
     * @param {StofNode} node
     * @returns {string}
     */
    exportStringFor(format, node) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            _assertClass(node, StofNode);
            const ret = wasm.stofdoc_exportStringFor(this.__wbg_ptr, ptr0, len0, node.__wbg_ptr);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Export this document to bytes using the format 'format'.
     * @param {string} format
     * @returns {any}
     */
    exportBytes(format) {
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_exportBytes(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Export a node to bytes using the format 'format'.
     * Some formats (like 'bstof') do not export for a singular node. It is up to the format
     * how it gets exported!
     * @param {string} format
     * @param {StofNode} node
     * @returns {any}
     */
    exportBytesFor(format, node) {
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_exportBytesFor(this.__wbg_ptr, ptr0, len0, node.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get all of the available libraries.
     * @returns {(string)[]}
     */
    availableLibraries() {
        const ret = wasm.stofdoc_availableLibraries(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Insert a custom JS library function.
     * @param {string} lib
     * @param {string} name
     * @param {any} func
     */
    insertLibFunc(lib, name, func) {
        const ptr0 = passStringToWasm0(lib, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        wasm.stofdoc_insertLibFunc(this.__wbg_ptr, ptr0, len0, ptr1, len1, func);
    }
    /**
     * Create all libraries from library functions.
     * Creates callable libraries out of all of the inserted custom library functions.
     * This is required before you can use the libraries within this document.
     */
    createLibs() {
        wasm.stofdoc_createLibs(this.__wbg_ptr);
    }
    /**
     * Get a value from this document from a path.
     * If the path points to a field, the value will be retrieved.
     * If the path points to a function, it will be called. Param is the function attribute 'get' if any.
     * @param {string} path
     * @returns {any}
     */
    get(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_get(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Find a field in this document with a path from the roots.
     * Path is dot separated indicating object names, with the field being the last name in the path.
     * Path will search down first (children) then up (parent). If root, will traverse into other roots.
     *
     * Ex. Field named 'message' on a root node: `fieldFromRoot('message')`
     *
     * Ex. Field named 'message' on the 'root' root node: `fieldFromRoot('root.message')`
     *
     * Ex. Field named 'message' on the child node at path 'root.child': `fieldFromRoot('root.child.message')`
     * @param {string} path
     * @returns {StofField | undefined}
     */
    fieldFromRoot(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_fieldFromRoot(this.__wbg_ptr, ptr0, len0);
        return ret === 0 ? undefined : StofField.__wrap(ret);
    }
    /**
     * Find a field in this document starting at a node.
     * Path is dot separated indicating object names, with the field being the last name in the path.
     * Path will search down first (children) then up (parent). If root, will traverse into other roots.
     * @param {string} path
     * @param {StofNode} node
     * @returns {StofField | undefined}
     */
    field(path, node) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_field(this.__wbg_ptr, ptr0, len0, node.__wbg_ptr);
        return ret === 0 ? undefined : StofField.__wrap(ret);
    }
    /**
     * Find a function in this document with a path from the roots.
     * Path is dot separated indicating object names, with the func being the last name in the path.
     * Path will search down first (children) then up (parent). If root, will traverse into other roots.
     * @param {string} path
     * @returns {StofFunc | undefined}
     */
    funcFromRoot(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_funcFromRoot(this.__wbg_ptr, ptr0, len0);
        return ret === 0 ? undefined : StofFunc.__wrap(ret);
    }
    /**
     * Find a function in this document starting at a node.
     * Path is dot separated indicating object names, with the func being the last name in the path.
     * Path will search down first (children) then up (parent). If root, will traverse into other roots.
     * @param {string} path
     * @param {StofNode} node
     * @returns {StofFunc | undefined}
     */
    func(path, node) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_func(this.__wbg_ptr, ptr0, len0, node.__wbg_ptr);
        return ret === 0 ? undefined : StofFunc.__wrap(ret);
    }
    /**
     * Call a function in this document at the given path.
     * @param {string} path
     * @param {any[]} params
     * @returns {any}
     */
    callFunc(path, params) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(params, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_callFunc(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Run this document, calling all #[main] functions.
     * @returns {string | undefined}
     */
    run() {
        const ret = wasm.stofdoc_run(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Run this node, calling all #[main] functions on or under this node.
     * @param {StofNode} node
     * @returns {string | undefined}
     */
    runAt(node) {
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_runAt(this.__wbg_ptr, node.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Main root.
     * This is the first root in the graph, commonly named 'root'.
     * @returns {StofNode | undefined}
     */
    mainRoot() {
        const ret = wasm.stofdoc_mainRoot(this.__wbg_ptr);
        return ret === 0 ? undefined : StofNode.__wrap(ret);
    }
    /**
     * Root by name.
     * @param {string} name
     * @returns {StofNode | undefined}
     */
    rootByName(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_rootByName(this.__wbg_ptr, ptr0, len0);
        return ret === 0 ? undefined : StofNode.__wrap(ret);
    }
    /**
     * Is a root?
     * @param {StofNode} node
     * @returns {boolean}
     */
    isRoot(node) {
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_isRoot(this.__wbg_ptr, node.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Roots.
     * @returns {(StofNode)[]}
     */
    roots() {
        const ret = wasm.stofdoc_roots(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Insert a new root node.
     * @param {string} name
     * @returns {StofNode}
     */
    insertRoot(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_insertRoot(this.__wbg_ptr, ptr0, len0);
        return StofNode.__wrap(ret);
    }
    /**
     * Insert a new node with a parent.
     * If the parent doesn't exist, this will create a root.
     * @param {string} name
     * @param {StofNode} parent
     * @returns {StofNode}
     */
    insertNode(name, parent) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(parent, StofNode);
        const ret = wasm.stofdoc_insertNode(this.__wbg_ptr, ptr0, len0, parent.__wbg_ptr);
        return StofNode.__wrap(ret);
    }
    /**
     * Insert a new node with a specific ID.
     * @param {string} name
     * @param {string} id
     * @param {StofNode} parent
     * @returns {StofNode}
     */
    insertNodeWithId(name, id, parent) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        _assertClass(parent, StofNode);
        const ret = wasm.stofdoc_insertNodeWithId(this.__wbg_ptr, ptr0, len0, ptr1, len1, parent.__wbg_ptr);
        return StofNode.__wrap(ret);
    }
    /**
     * Remove a node.
     * Removes all data on this node, deleting from the graph if this is the only node
     * it is referenced on.
     * @param {StofNode} node
     * @returns {boolean}
     */
    removeNode(node) {
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_removeNode(this.__wbg_ptr, node.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Get all children of a node, on all children, grandchildren, etc...
     * @param {StofNode} node
     * @returns {(StofNode)[]}
     */
    allChildren(node) {
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_allChildren(this.__wbg_ptr, node.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Create new data on a node.
     * @param {StofNode} node
     * @param {any} value
     * @returns {StofData}
     */
    createData(node, value) {
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_createData(this.__wbg_ptr, node.__wbg_ptr, value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return StofData.__wrap(ret[0]);
    }
    /**
     * Create new data on a node with an ID.
     * @param {StofNode} node
     * @param {string} id
     * @param {any} value
     * @returns {StofData}
     */
    createDataWithId(node, id, value) {
        _assertClass(node, StofNode);
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdoc_createDataWithId(this.__wbg_ptr, node.__wbg_ptr, ptr0, len0, value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return StofData.__wrap(ret[0]);
    }
    /**
     * Put data onto a node.
     * @param {StofNode} node
     * @param {StofData} data
     * @returns {boolean}
     */
    putData(node, data) {
        _assertClass(node, StofNode);
        _assertClass(data, StofData);
        const ret = wasm.stofdoc_putData(this.__wbg_ptr, node.__wbg_ptr, data.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Remove data from everywhere in this document.
     * @param {StofData} data
     * @returns {boolean}
     */
    removeData(data) {
        _assertClass(data, StofData);
        const ret = wasm.stofdoc_removeData(this.__wbg_ptr, data.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Remove data from a specific node in this document.
     * @param {StofData} data
     * @param {StofNode} node
     * @returns {boolean}
     */
    removeDataFrom(data, node) {
        _assertClass(data, StofData);
        _assertClass(node, StofNode);
        const ret = wasm.stofdoc_removeDataFrom(this.__wbg_ptr, data.__wbg_ptr, node.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Flush node deadpool.
     * @returns {any[]}
     */
    flush_node_deadpool() {
        const ret = wasm.stofdoc_flush_node_deadpool(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Flush data deadpool.
     * @returns {any[]}
     */
    flush_data_deadpool() {
        const ret = wasm.stofdoc_flush_data_deadpool(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Flush nodes.
     * Collect dirty nodes for validation.
     * For no limit, pass -1.
     * @param {number} limit
     * @returns {(StofNode)[]}
     */
    flush_nodes(limit) {
        const ret = wasm.stofdoc_flush_nodes(this.__wbg_ptr, limit);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Flush data.
     * Collect dirty data for validation.
     * For no limit, pass -1.
     * @param {number} limit
     * @returns {(StofData)[]}
     */
    flush_data(limit) {
        const ret = wasm.stofdoc_flush_data(this.__wbg_ptr, limit);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}

const StofFieldFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stoffield_free(ptr >>> 0, 1));
/**
 * JS Stof Field.
 */
export class StofField {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StofField.prototype);
        obj.__wbg_ptr = ptr;
        StofFieldFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StofFieldFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stoffield_free(ptr, 0);
    }
    /**
     * New field.
     * Does not insert into the document, but needs the document for JsValue -> SVal.
     * @param {StofDoc} doc
     * @param {string} name
     * @param {any} value
     */
    constructor(doc, name, value) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stoffield_new(doc.__wbg_ptr, ptr0, len0, value);
        this.__wbg_ptr = ret >>> 0;
        StofFieldFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * From a data reference.
     * @param {StofDoc} doc
     * @param {StofData} data
     * @returns {StofField}
     */
    static fromData(doc, data) {
        _assertClass(doc, StofDoc);
        _assertClass(data, StofData);
        const ret = wasm.stoffield_fromData(doc.__wbg_ptr, data.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return StofField.__wrap(ret[0]);
    }
    /**
     * Data reference.
     * @returns {StofData}
     */
    data() {
        const ret = wasm.stoffield_data(this.__wbg_ptr);
        return StofData.__wrap(ret);
    }
    /**
     * Name of this field.
     * @returns {string}
     */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stoffield_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Value of this field.
     * @returns {any}
     */
    value() {
        const ret = wasm.stoffield_value(this.__wbg_ptr);
        return ret;
    }
    /**
     * Set the value of this field.
     * If this field exists within the document, it will set the value in the document as well.
     * @param {StofDoc} doc
     * @param {any} value
     */
    setValue(doc, value) {
        _assertClass(doc, StofDoc);
        wasm.stoffield_setValue(this.__wbg_ptr, doc.__wbg_ptr, value);
    }
    /**
     * Attach this field to a node within the document.
     * @param {StofDoc} doc
     * @param {StofNode} node
     */
    attach(doc, node) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        wasm.stoffield_attach(this.__wbg_ptr, doc.__wbg_ptr, node.__wbg_ptr);
    }
    /**
     * Remove this field from the document everywhere.
     * @param {StofDoc} doc
     */
    remove(doc) {
        _assertClass(doc, StofDoc);
        wasm.stoffield_remove(this.__wbg_ptr, doc.__wbg_ptr);
    }
    /**
     * Remove this field from a specific node.
     * If this node is the only one that references the field, the field will be removed from the doc.
     * @param {StofDoc} doc
     * @param {StofNode} node
     */
    removeFrom(doc, node) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        wasm.stoffield_removeFrom(this.__wbg_ptr, doc.__wbg_ptr, node.__wbg_ptr);
    }
    /**
     * Get all fields on a node.
     * @param {StofDoc} doc
     * @param {StofNode} node
     * @returns {(StofField)[]}
     */
    static fields(doc, node) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        const ret = wasm.stoffield_fields(doc.__wbg_ptr, node.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get an adjacent field to this field in the document from a dot separated path.
     * @param {StofDoc} doc
     * @param {string} path
     * @returns {StofField | undefined}
     */
    adjacent(doc, path) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stoffield_adjacent(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        return ret === 0 ? undefined : StofField.__wrap(ret);
    }
    /**
     * Get a specific field from a dot separated path, starting at the root.
     * @param {StofDoc} doc
     * @param {string} path
     * @returns {StofField | undefined}
     */
    static fieldFromRoot(doc, path) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stoffield_fieldFromRoot(doc.__wbg_ptr, ptr0, len0);
        return ret === 0 ? undefined : StofField.__wrap(ret);
    }
    /**
     * Get a specific field from a dot separated path, starting at a node.
     * @param {StofDoc} doc
     * @param {string} path
     * @param {StofNode} node
     * @returns {StofField | undefined}
     */
    static field(doc, path, node) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(node, StofNode);
        const ret = wasm.stoffield_field(doc.__wbg_ptr, ptr0, len0, node.__wbg_ptr);
        return ret === 0 ? undefined : StofField.__wrap(ret);
    }
}

const StofFuncFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stoffunc_free(ptr >>> 0, 1));
/**
 * JS Stof Func.
 */
export class StofFunc {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StofFunc.prototype);
        obj.__wbg_ptr = ptr;
        StofFuncFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StofFuncFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stoffunc_free(ptr, 0);
    }
    /**
     * From a data reference.
     * @param {StofDoc} doc
     * @param {StofData} data
     * @returns {StofFunc}
     */
    static fromData(doc, data) {
        _assertClass(doc, StofDoc);
        _assertClass(data, StofData);
        const ret = wasm.stoffunc_fromData(doc.__wbg_ptr, data.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return StofFunc.__wrap(ret[0]);
    }
    /**
     * Data reference.
     * @returns {StofData}
     */
    data() {
        const ret = wasm.stoffunc_data(this.__wbg_ptr);
        return StofData.__wrap(ret);
    }
    /**
     * Name of this func.
     * @returns {string}
     */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stoffunc_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Return type of this func.
     * @returns {string}
     */
    returnType() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stoffunc_returnType(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Parameters of this func.
     * @returns {(StofFuncParam)[]}
     */
    parameters() {
        const ret = wasm.stoffunc_parameters(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Attach this func to a node within the document.
     * @param {StofDoc} doc
     * @param {StofNode} node
     */
    attach(doc, node) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        wasm.stoffunc_attach(this.__wbg_ptr, doc.__wbg_ptr, node.__wbg_ptr);
    }
    /**
     * Remove this func from the document everywhere.
     * @param {StofDoc} doc
     */
    remove(doc) {
        _assertClass(doc, StofDoc);
        wasm.stoffunc_remove(this.__wbg_ptr, doc.__wbg_ptr);
    }
    /**
     * Remove this func from a specific node.
     * If this node is the only one that references the func, the func will be removed from the doc.
     * @param {StofDoc} doc
     * @param {StofNode} node
     */
    removeFrom(doc, node) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        wasm.stoffunc_removeFrom(this.__wbg_ptr, doc.__wbg_ptr, node.__wbg_ptr);
    }
    /**
     * Get all funcs on a node.
     * @param {StofDoc} doc
     * @param {StofNode} node
     * @returns {(StofFunc)[]}
     */
    static funcs(doc, node) {
        _assertClass(doc, StofDoc);
        _assertClass(node, StofNode);
        const ret = wasm.stoffunc_funcs(doc.__wbg_ptr, node.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get an adjacent func to this func in the document from a dot separated path.
     * @param {StofDoc} doc
     * @param {string} path
     * @returns {StofFunc | undefined}
     */
    adjacent(doc, path) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stoffunc_adjacent(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        return ret === 0 ? undefined : StofFunc.__wrap(ret);
    }
    /**
     * Get a specific func from a dot separated path, starting at the root.
     * @param {StofDoc} doc
     * @param {string} path
     * @returns {StofFunc | undefined}
     */
    static funcFromRoot(doc, path) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stoffunc_funcFromRoot(doc.__wbg_ptr, ptr0, len0);
        return ret === 0 ? undefined : StofFunc.__wrap(ret);
    }
    /**
     * Get a specific func from a dot separated path, starting at a node.
     * @param {StofDoc} doc
     * @param {string} path
     * @param {StofNode} node
     * @returns {StofFunc | undefined}
     */
    static func(doc, path, node) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(node, StofNode);
        const ret = wasm.stoffunc_func(doc.__wbg_ptr, ptr0, len0, node.__wbg_ptr);
        return ret === 0 ? undefined : StofFunc.__wrap(ret);
    }
    /**
     * Call this function.
     * @param {StofDoc} doc
     * @param {any[]} params
     * @returns {any}
     */
    call(doc, params) {
        _assertClass(doc, StofDoc);
        const ptr0 = passArrayJsValueToWasm0(params, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stoffunc_call(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const StofFuncParamFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stoffuncparam_free(ptr >>> 0, 1));
/**
 * Stof Func param interface.
 */
export class StofFuncParam {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StofFuncParam.prototype);
        obj.__wbg_ptr = ptr;
        StofFuncParamFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StofFuncParamFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stoffuncparam_free(ptr, 0);
    }
    /**
     * Name.
     * @returns {string}
     */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stoffuncparam_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Type.
     * @returns {string}
     */
    type() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stoffuncparam_type(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Has a default value?
     * @returns {boolean}
     */
    hasDefault() {
        const ret = wasm.stoffuncparam_hasDefault(this.__wbg_ptr);
        return ret !== 0;
    }
}

const StofLibFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stoflib_free(ptr >>> 0, 1));
/**
 * JS Stof Lib.
 */
export class StofLib {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StofLibFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stoflib_free(ptr, 0);
    }
    /**
     * Create a new StofLib.
     * @param {string} scope
     */
    constructor(scope) {
        const ptr0 = passStringToWasm0(scope, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stoflib_new(ptr0, len0);
        this.__wbg_ptr = ret >>> 0;
        StofLibFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Name of this library.
     * This is how it will be referenced from within Stof.
     * @returns {string}
     */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stoflib_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Load a library into a document.
     * @param {StofDoc} doc
     * @param {StofLib} lib
     */
    static load(doc, lib) {
        _assertClass(doc, StofDoc);
        _assertClass(lib, StofLib);
        var ptr0 = lib.__destroy_into_raw();
        wasm.stoflib_load(doc.__wbg_ptr, ptr0);
    }
}

const StofNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stofnode_free(ptr >>> 0, 1));
/**
 * Stof Node.
 */
export class StofNode {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StofNode.prototype);
        obj.__wbg_ptr = ptr;
        StofNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StofNodeFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stofnode_free(ptr, 0);
    }
    /**
     * ID constructor.
     * @param {string} id
     */
    constructor(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofdata_new(ptr0, len0);
        this.__wbg_ptr = ret >>> 0;
        StofNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Path constructor.
     * @param {StofDoc} doc
     * @param {string} path
     * @returns {StofNode | undefined}
     */
    static fromPath(doc, path) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofnode_fromPath(doc.__wbg_ptr, ptr0, len0);
        return ret === 0 ? undefined : StofNode.__wrap(ret);
    }
    /**
     * Path from a starting point constructor.
     * @param {StofDoc} doc
     * @param {string} path
     * @param {StofNode} start
     * @returns {StofNode | undefined}
     */
    static fromPathStart(doc, path, start) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(start, StofNode);
        const ret = wasm.stofnode_fromPathStart(doc.__wbg_ptr, ptr0, len0, start.__wbg_ptr);
        return ret === 0 ? undefined : StofNode.__wrap(ret);
    }
    /**
     * Get the ID of this node reference.
     * @returns {string}
     */
    id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stofnode_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Invalidate this node with a symbol.
     * @param {StofDoc} doc
     * @param {string} symbol
     * @returns {boolean}
     */
    invalidate(doc, symbol) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(symbol, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofnode_invalidate(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Invalidate all on this node.
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    invalidateAll(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_invalidateAll(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Dirty?
     * @param {StofDoc} doc
     * @param {string} symbol
     * @returns {boolean}
     */
    dirty(doc, symbol) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(symbol, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofnode_dirty(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Any dirty symbols?
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    anyDirty(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_anyDirty(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Validate this node with the symbol.
     * @param {StofDoc} doc
     * @param {string} symbol
     * @returns {boolean}
     */
    validate(doc, symbol) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(symbol, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofnode_validate(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Validate all for this node.
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    validate_all(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_validate_all(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Root node for this reference.
     * @param {StofDoc} doc
     * @returns {StofNode | undefined}
     */
    root(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_root(this.__wbg_ptr, doc.__wbg_ptr);
        return ret === 0 ? undefined : StofNode.__wrap(ret);
    }
    /**
     * Exists within the document?
     * @param {StofDoc} doc
     * @returns {boolean}
     */
    exists(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_exists(this.__wbg_ptr, doc.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Is a child of the 'parent' node?
     * Returns true if this and parent are equal.
     * Returns true if this node is a granchild or below.
     * @param {StofDoc} doc
     * @param {StofNode} parent
     * @returns {boolean}
     */
    isChildOf(doc, parent) {
        _assertClass(doc, StofDoc);
        _assertClass(parent, StofNode);
        const ret = wasm.stofnode_isChildOf(this.__wbg_ptr, doc.__wbg_ptr, parent.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Is an immediate child of 'parent'?
     * Will return false if this node is a grandchild or below.
     * @param {StofDoc} doc
     * @param {StofNode} parent
     * @returns {boolean}
     */
    isImmediateChildOf(doc, parent) {
        _assertClass(doc, StofDoc);
        _assertClass(parent, StofNode);
        const ret = wasm.stofnode_isImmediateChildOf(this.__wbg_ptr, doc.__wbg_ptr, parent.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return the named path of this node.
     * Path is '/' separated and starts at this nodes root.
     * @param {StofDoc} doc
     * @returns {string}
     */
    path(doc) {
        let deferred1_0;
        let deferred1_1;
        try {
            _assertClass(doc, StofDoc);
            const ret = wasm.stofnode_path(this.__wbg_ptr, doc.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Return the ID path of this node.
     * @param {StofDoc} doc
     * @returns {(string)[]}
     */
    idPath(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_idPath(this.__wbg_ptr, doc.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Distance to another node in the document.
     * @param {StofDoc} doc
     * @param {StofNode} other
     * @returns {number}
     */
    distanceTo(doc, other) {
        _assertClass(doc, StofDoc);
        _assertClass(other, StofNode);
        const ret = wasm.stofnode_distanceTo(this.__wbg_ptr, doc.__wbg_ptr, other.__wbg_ptr);
        return ret;
    }
    /**
     * Name of this node.
     * @param {StofDoc} doc
     * @returns {string | undefined}
     */
    name(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_name(this.__wbg_ptr, doc.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Parent of this node.
     * @param {StofDoc} doc
     * @returns {StofNode | undefined}
     */
    parent(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_parent(this.__wbg_ptr, doc.__wbg_ptr);
        return ret === 0 ? undefined : StofNode.__wrap(ret);
    }
    /**
     * Children of this node.
     * @param {StofDoc} doc
     * @returns {(StofNode)[]}
     */
    children(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_children(this.__wbg_ptr, doc.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Data on this node.
     * @param {StofDoc} doc
     * @returns {(StofData)[]}
     */
    data(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_data(this.__wbg_ptr, doc.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * All data on all children nodes.
     * @param {StofDoc} doc
     * @returns {(StofData)[]}
     */
    allData(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_allData(this.__wbg_ptr, doc.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Has data?
     * @param {StofDoc} doc
     * @param {StofData} data
     * @returns {boolean}
     */
    hasData(doc, data) {
        _assertClass(doc, StofDoc);
        _assertClass(data, StofData);
        const ret = wasm.stofnode_hasData(this.__wbg_ptr, doc.__wbg_ptr, data.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Data on this node with an ID that has the prefix 'prefix'.
     * @param {StofDoc} doc
     * @param {string} prefix
     * @returns {(StofData)[]}
     */
    prefixData(doc, prefix) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(prefix, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofnode_prefixData(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v2;
    }
    /**
     * All data on all children nodes with an ID that has the prefix 'prefix'.
     * @param {StofDoc} doc
     * @param {string} prefix
     * @returns {(StofData)[]}
     */
    allPrefixData(doc, prefix) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(prefix, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofnode_allPrefixData(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0);
        var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v2;
    }
    /**
     * Create some abstract data on this node.
     * @param {StofDoc} doc
     * @param {any} value
     * @returns {StofData}
     */
    createData(doc, value) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_createData(this.__wbg_ptr, doc.__wbg_ptr, value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return StofData.__wrap(ret[0]);
    }
    /**
     * Create a new field on this node.
     * @param {StofDoc} doc
     * @param {string} name
     * @param {any} value
     * @returns {StofField}
     */
    createField(doc, name, value) {
        _assertClass(doc, StofDoc);
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stofnode_createField(this.__wbg_ptr, doc.__wbg_ptr, ptr0, len0, value);
        return StofField.__wrap(ret);
    }
    /**
     * JSON value of this node as a whole.
     * Can use this to store this value in an external place.
     * @param {StofDoc} doc
     * @returns {any}
     */
    to_json(doc) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_to_json(this.__wbg_ptr, doc.__wbg_ptr);
        return ret;
    }
    /**
     * Loat a JSON representation of a node into a document.
     * Can use this to load nodes from an external place.
     * @param {StofDoc} doc
     * @param {any} json
     * @returns {boolean}
     */
    static from_json(doc, json) {
        _assertClass(doc, StofDoc);
        const ret = wasm.stofnode_from_json(doc.__wbg_ptr, json);
        return ret !== 0;
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_add_0d9e99fb9c2d2cc5 = function(arg0, arg1) {
        const ret = arg0.add(arg1);
        return ret;
    };
    imports.wbg.__wbg_buffer_61b7ce01341d7f88 = function(arg0) {
        const ret = arg0.buffer;
        return ret;
    };
    imports.wbg.__wbg_call_3b770f0d6eb4720e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.call(arg1, arg2, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_500db948e69c7330 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_9bd6f269d4835e33 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.call(arg1, arg2, arg3, arg4);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_b0d8e36992d9900d = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_crypto_ed58b8e10a292839 = function(arg0) {
        const ret = arg0.crypto;
        return ret;
    };
    imports.wbg.__wbg_done_f22c1561fa919baa = function(arg0) {
        const ret = arg0.done;
        return ret;
    };
    imports.wbg.__wbg_entries_4f2bb9b0d701c0f6 = function(arg0) {
        const ret = Object.entries(arg0);
        return ret;
    };
    imports.wbg.__wbg_from_d68eaa96dba25449 = function(arg0) {
        const ret = Array.from(arg0);
        return ret;
    };
    imports.wbg.__wbg_getRandomValues_bcb4912f16000dc4 = function() { return handleError(function (arg0, arg1) {
        arg0.getRandomValues(arg1);
    }, arguments) };
    imports.wbg.__wbg_get_9aa3dff3f0266054 = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return ret;
    };
    imports.wbg.__wbg_get_bbccf8970793c087 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.get(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getwithrefkey_1dc361bd10053bfe = function(arg0, arg1) {
        const ret = arg0[arg1];
        return ret;
    };
    imports.wbg.__wbg_instanceof_ArrayBuffer_670ddde44cdb2602 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof ArrayBuffer;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Map_98ecb30afec5acdb = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Map;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Uint8Array_28af5bc19d6acad8 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Uint8Array;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_isArray_1ba11a930108ec51 = function(arg0) {
        const ret = Array.isArray(arg0);
        return ret;
    };
    imports.wbg.__wbg_isSafeInteger_12f5549b2fca23f4 = function(arg0) {
        const ret = Number.isSafeInteger(arg0);
        return ret;
    };
    imports.wbg.__wbg_iterator_23604bb983791576 = function() {
        const ret = Symbol.iterator;
        return ret;
    };
    imports.wbg.__wbg_length_65d1cd11729ced11 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_d65cf0786bfc5739 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_msCrypto_0a36e2ec3a343d26 = function(arg0) {
        const ret = arg0.msCrypto;
        return ret;
    };
    imports.wbg.__wbg_new_0f1bd659dcd47068 = function(arg0) {
        const ret = new Set(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_254fa9eac11932ae = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_new_3ff5b33b1ce712df = function(arg0) {
        const ret = new Uint8Array(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_688846f374351c92 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_bc96c6a1c0786643 = function() {
        const ret = new Map();
        return ret;
    };
    imports.wbg.__wbg_newnoargs_fd9e4bf8be2bc16d = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_ba35896968751d91 = function(arg0, arg1, arg2) {
        const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_newwithlength_34ce8f1051e74449 = function(arg0) {
        const ret = new Uint8Array(arg0 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_next_01dd9234a5bf6d05 = function() { return handleError(function (arg0) {
        const ret = arg0.next();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_next_137428deb98342b0 = function(arg0) {
        const ret = arg0.next;
        return ret;
    };
    imports.wbg.__wbg_node_02999533c4ea02e3 = function(arg0) {
        const ret = arg0.node;
        return ret;
    };
    imports.wbg.__wbg_process_5c1d670bc53614b8 = function(arg0) {
        const ret = arg0.process;
        return ret;
    };
    imports.wbg.__wbg_push_6edad0df4b546b2c = function(arg0, arg1) {
        const ret = arg0.push(arg1);
        return ret;
    };
    imports.wbg.__wbg_randomFillSync_ab2cfe79ebbf2740 = function() { return handleError(function (arg0, arg1) {
        arg0.randomFillSync(arg1);
    }, arguments) };
    imports.wbg.__wbg_require_79b1e9274cde3c87 = function() { return handleError(function () {
        const ret = module.require;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_set_1d80752d0d5f0b21 = function(arg0, arg1, arg2) {
        arg0[arg1 >>> 0] = arg2;
    };
    imports.wbg.__wbg_set_23d69db4e5c66a6e = function(arg0, arg1, arg2) {
        arg0.set(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
        arg0[arg1] = arg2;
    };
    imports.wbg.__wbg_set_76818dc3c59a63d5 = function(arg0, arg1, arg2) {
        const ret = arg0.set(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_0be7472e492ad3e3 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_1a6eb482d12c9bfb = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_1dc398a895c82351 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_ae1c80c7eea8d64a = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_stofdata_new = function(arg0) {
        const ret = StofData.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_stoffield_new = function(arg0) {
        const ret = StofField.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_stoffunc_new = function(arg0) {
        const ret = StofFunc.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_stoffuncparam_new = function(arg0) {
        const ret = StofFuncParam.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_stofnode_new = function(arg0) {
        const ret = StofNode.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_subarray_46adeb9b86949d12 = function(arg0, arg1, arg2) {
        const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_value_4c32fd138a88eee2 = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbg_versions_c71aa1626a93e0a1 = function(arg0) {
        const ret = arg0.versions;
        return ret;
    };
    imports.wbg.__wbindgen_as_number = function(arg0) {
        const ret = +arg0;
        return ret;
    };
    imports.wbg.__wbindgen_bigint_from_i64 = function(arg0) {
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
        const ret = BigInt.asUintN(64, arg0);
        return ret;
    };
    imports.wbg.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
        const v = arg1;
        const ret = typeof(v) === 'bigint' ? v : undefined;
        getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = arg0;
        const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        return ret;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbindgen_in = function(arg0, arg1) {
        const ret = arg0 in arg1;
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_2;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_is_array = function(arg0) {
        const ret = Array.isArray(arg0);
        return ret;
    };
    imports.wbg.__wbindgen_is_bigint = function(arg0) {
        const ret = typeof(arg0) === 'bigint';
        return ret;
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(arg0) === 'function';
        return ret;
    };
    imports.wbg.__wbindgen_is_null = function(arg0) {
        const ret = arg0 === null;
        return ret;
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = arg0;
        const ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbindgen_is_string = function(arg0) {
        const ret = typeof(arg0) === 'string';
        return ret;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = arg0 === arg1;
        return ret;
    };
    imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
        const ret = arg0 == arg1;
        return ret;
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return ret;
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'number' ? obj : undefined;
        getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('stof_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
