/* tslint:disable */
/* eslint-disable */
/**
 * Stof Data.
 */
export class StofData {
  free(): void;
  /**
   * JSON data constructor.
   * Will turn value into serde_json::Value, then create an SData, putting it into the document at 'node'.
   * Only works if node is already in the document.
   */
  constructor(doc: StofDoc, node: StofNode, value: any);
  /**
   * Construct a new StofData with an ID and a value.
   */
  static newWithId(doc: StofDoc, node: StofNode, id: string, value: any): StofData;
  /**
   * Remove this data from every place within the document.
   */
  remove(doc: StofDoc): boolean;
  /**
   * Remove this data from a specific node in the document.
   */
  removeFrom(doc: StofDoc, node: StofNode): boolean;
  /**
   * ID constructor.
   */
  static new(id: string): StofData;
  /**
   * Get the ID of this reference.
   */
  id(): string;
  /**
   * Invalidate this data with a symbol.
   */
  invalidate(doc: StofDoc, symbol: string): boolean;
  /**
   * Invalidate value on this data.
   */
  invalidateValue(doc: StofDoc): boolean;
  /**
   * Dirty?
   */
  dirty(doc: StofDoc, symbol: string): boolean;
  /**
   * Any dirty symbols?
   */
  anyDirty(doc: StofDoc): boolean;
  /**
   * Validate this data with the symbol.
   */
  validate(doc: StofDoc, symbol: string): boolean;
  /**
   * Validate value for this data.
   */
  validate_all(doc: StofDoc): boolean;
  /**
   * Exists?
   */
  exists(doc: StofDoc): boolean;
  /**
   * Nodes that contain this data.
   * Data can exist on several nodes at once.
   */
  nodes(doc: StofDoc): (StofNode)[];
  /**
   * Try getting the JSON value of this data.
   * Will only work if the value of this data can be deserialized into serde_json::Value.
   */
  getValue(doc: StofDoc): any;
  /**
   * Try setting a JSON value for this data.
   */
  setValue(doc: StofDoc, value: any): boolean;
  /**
   * JSON value of this data as a whole.
   * Can use this to store this value in an external place.
   */
  to_json(doc: StofDoc): any;
  /**
   * Loat a JSON representation of a data into a document.
   * Can use this to load data from an external place.
   */
  static from_json(doc: StofDoc, json: any): boolean;
}
/**
 * JS Stof Document Interface.
 */
export class StofDoc {
  free(): void;
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
   */
  constructor(name: string, src: string, format: string);
  /**
   * Construct a new StofDoc using a JS object.
   * This will convert the object into a serde_json::Value before creating a document out of it, capturing it's fields, sub-objects, etc...
   */
  static js(name: string, jsobj: any): StofDoc;
  /**
   * Construct a new StofDoc using bytes and a format.
   */
  static bytes(name: string, bytes: any, format: string): StofDoc;
  /**
   * Get the ID of this document.
   * This is a unique string ID, generated with nanoid. Can be used for storage, etc...
   */
  id(): string;
  /**
   * Get the name of this document.
   */
  name(): string;
  /**
   * Set the name of this document.
   */
  setName(name: string): void;
  /**
   * Version of this document.
   */
  version(): string;
  /**
   * Get all of the available formats.
   */
  availableFormats(): (string)[];
  /**
   * Get the content type for a format.
   */
  formatContentType(format: string): string | undefined;
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
   */
  headerImport(format: string, content_type: string, bytes: any, as_name: string): boolean;
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
   */
  stringImport(format: string, src: string, as_name: string): boolean;
  /**
   * File import.
   * Used for importing/parsing files into this document with the given format.
   *
   * By default, the "fs" (file system) library is not included, so you'll need to implement the following functions yourself:
   * - "fs.read" with one path (str) parameter `doc.insertLibFunc('fs', 'read', (path: string):string => {...}`
   * - "fs.read_blob" with one path (str) parameter `doc.insertLibFunc('fs', 'read_blob', (path: string):Uint8Array => {...}`
   * - "fs.write" with two parameters `doc.insertLibFunc('fs', 'write', (path: string, contents: string) => {...}`
   * - "fs.write_blob" with two parameters `doc.insertLibFunc('fs', 'write_blob', (path: string, contents: Uint8Array) => {...}`
   */
  fileImport(format: string, path: string, extension: string, as_name: string): boolean;
  /**
   * Export this document to a string using the format 'format'.
   */
  exportString(format: string): string;
  /**
   * Export a node to a string using the format 'format'.
   */
  exportStringFor(format: string, node: StofNode): string;
  /**
   * Export this document to bytes using the format 'format'.
   */
  exportBytes(format: string): any;
  /**
   * Export a node to bytes using the format 'format'.
   * Some formats (like 'bstof') do not export for a singular node. It is up to the format
   * how it gets exported!
   */
  exportBytesFor(format: string, node: StofNode): any;
  /**
   * Get all of the available libraries.
   */
  availableLibraries(): (string)[];
  /**
   * Insert a custom JS library function.
   */
  insertLibFunc(lib: string, name: string, func: any): void;
  /**
   * Create all libraries from library functions.
   * Creates callable libraries out of all of the inserted custom library functions.
   * This is required before you can use the libraries within this document.
   */
  createLibs(): void;
  /**
   * Get a value from this document from a path.
   * If the path points to a field, the value will be retrieved.
   * If the path points to a function, it will be called. Param is the function attribute 'get' if any.
   */
  get(path: string): any;
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
   */
  fieldFromRoot(path: string): StofField | undefined;
  /**
   * Find a field in this document starting at a node.
   * Path is dot separated indicating object names, with the field being the last name in the path.
   * Path will search down first (children) then up (parent). If root, will traverse into other roots.
   */
  field(path: string, node: StofNode): StofField | undefined;
  /**
   * Find a function in this document with a path from the roots.
   * Path is dot separated indicating object names, with the func being the last name in the path.
   * Path will search down first (children) then up (parent). If root, will traverse into other roots.
   */
  funcFromRoot(path: string): StofFunc | undefined;
  /**
   * Find a function in this document starting at a node.
   * Path is dot separated indicating object names, with the func being the last name in the path.
   * Path will search down first (children) then up (parent). If root, will traverse into other roots.
   */
  func(path: string, node: StofNode): StofFunc | undefined;
  /**
   * Call a function in this document at the given path.
   */
  callFunc(path: string, params: any[]): any;
  /**
   * Run this document, calling all #[main] functions.
   */
  run(): void;
  /**
   * Run this node, calling all #[main] functions on or under this node.
   */
  runAt(node: StofNode): void;
  /**
   * Main root.
   * This is the first root in the graph, commonly named 'root'.
   */
  mainRoot(): StofNode | undefined;
  /**
   * Root by name.
   */
  rootByName(name: string): StofNode | undefined;
  /**
   * Is a root?
   */
  isRoot(node: StofNode): boolean;
  /**
   * Roots.
   */
  roots(): (StofNode)[];
  /**
   * Insert a new root node.
   */
  insertRoot(name: string): StofNode;
  /**
   * Insert a new node with a parent.
   * If the parent doesn't exist, this will create a root.
   */
  insertNode(name: string, parent: StofNode): StofNode;
  /**
   * Insert a new node with a specific ID.
   */
  insertNodeWithId(name: string, id: string, parent: StofNode): StofNode;
  /**
   * Remove a node.
   * Removes all data on this node, deleting from the graph if this is the only node
   * it is referenced on.
   */
  removeNode(node: StofNode): boolean;
  /**
   * Get all children of a node, on all children, grandchildren, etc...
   */
  allChildren(node: StofNode): (StofNode)[];
  /**
   * Create new data on a node.
   */
  createData(node: StofNode, value: any): StofData;
  /**
   * Create new data on a node with an ID.
   */
  createDataWithId(node: StofNode, id: string, value: any): StofData;
  /**
   * Put data onto a node.
   */
  putData(node: StofNode, data: StofData): boolean;
  /**
   * Remove data from everywhere in this document.
   */
  removeData(data: StofData): boolean;
  /**
   * Remove data from a specific node in this document.
   */
  removeDataFrom(data: StofData, node: StofNode): boolean;
  /**
   * Flush node deadpool.
   */
  flush_node_deadpool(): any[];
  /**
   * Flush data deadpool.
   */
  flush_data_deadpool(): any[];
  /**
   * Flush nodes.
   * Collect dirty nodes for validation.
   * For no limit, pass -1.
   */
  flush_nodes(limit: number): (StofNode)[];
  /**
   * Flush data.
   * Collect dirty data for validation.
   * For no limit, pass -1.
   */
  flush_data(limit: number): (StofData)[];
}
/**
 * JS Stof Field.
 */
export class StofField {
  free(): void;
  /**
   * New field.
   * Does not insert into the document, but needs the document for JsValue -> SVal.
   */
  constructor(doc: StofDoc, name: string, value: any);
  /**
   * From a data reference.
   */
  static fromData(doc: StofDoc, data: StofData): StofField;
  /**
   * Data reference.
   */
  data(): StofData;
  /**
   * Name of this field.
   */
  name(): string;
  /**
   * Value of this field.
   */
  value(): any;
  /**
   * Set the value of this field.
   * If this field exists within the document, it will set the value in the document as well.
   */
  setValue(doc: StofDoc, value: any): void;
  /**
   * Attach this field to a node within the document.
   */
  attach(doc: StofDoc, node: StofNode): void;
  /**
   * Remove this field from the document everywhere.
   */
  remove(doc: StofDoc): void;
  /**
   * Remove this field from a specific node.
   * If this node is the only one that references the field, the field will be removed from the doc.
   */
  removeFrom(doc: StofDoc, node: StofNode): void;
  /**
   * Get all fields on a node.
   */
  static fields(doc: StofDoc, node: StofNode): (StofField)[];
  /**
   * Get an adjacent field to this field in the document from a dot separated path.
   */
  adjacent(doc: StofDoc, path: string): StofField | undefined;
  /**
   * Get a specific field from a dot separated path, starting at the root.
   */
  static fieldFromRoot(doc: StofDoc, path: string): StofField | undefined;
  /**
   * Get a specific field from a dot separated path, starting at a node.
   */
  static field(doc: StofDoc, path: string, node: StofNode): StofField | undefined;
}
/**
 * JS Stof Func.
 */
export class StofFunc {
  private constructor();
  free(): void;
  /**
   * From a data reference.
   */
  static fromData(doc: StofDoc, data: StofData): StofFunc;
  /**
   * Data reference.
   */
  data(): StofData;
  /**
   * Name of this func.
   */
  name(): string;
  /**
   * Return type of this func.
   */
  returnType(): string;
  /**
   * Parameters of this func.
   */
  parameters(): (StofFuncParam)[];
  /**
   * Attach this func to a node within the document.
   */
  attach(doc: StofDoc, node: StofNode): void;
  /**
   * Remove this func from the document everywhere.
   */
  remove(doc: StofDoc): void;
  /**
   * Remove this func from a specific node.
   * If this node is the only one that references the func, the func will be removed from the doc.
   */
  removeFrom(doc: StofDoc, node: StofNode): void;
  /**
   * Get all funcs on a node.
   */
  static funcs(doc: StofDoc, node: StofNode): (StofFunc)[];
  /**
   * Get an adjacent func to this func in the document from a dot separated path.
   */
  adjacent(doc: StofDoc, path: string): StofFunc | undefined;
  /**
   * Get a specific func from a dot separated path, starting at the root.
   */
  static funcFromRoot(doc: StofDoc, path: string): StofFunc | undefined;
  /**
   * Get a specific func from a dot separated path, starting at a node.
   */
  static func(doc: StofDoc, path: string, node: StofNode): StofFunc | undefined;
  /**
   * Call this function.
   */
  call(doc: StofDoc, params: any[]): any;
}
/**
 * Stof Func param interface.
 */
export class StofFuncParam {
  private constructor();
  free(): void;
  /**
   * Name.
   */
  name(): string;
  /**
   * Type.
   */
  type(): string;
  /**
   * Has a default value?
   */
  hasDefault(): boolean;
}
/**
 * JS Stof Lib.
 */
export class StofLib {
  free(): void;
  /**
   * Create a new StofLib.
   */
  constructor(scope: string);
  /**
   * Name of this library.
   * This is how it will be referenced from within Stof.
   */
  name(): string;
  /**
   * Load a library into a document.
   */
  static load(doc: StofDoc, lib: StofLib): void;
}
/**
 * Stof Node.
 */
export class StofNode {
  free(): void;
  /**
   * ID constructor.
   */
  constructor(id: string);
  /**
   * Path constructor.
   */
  static fromPath(doc: StofDoc, path: string): StofNode | undefined;
  /**
   * Path from a starting point constructor.
   */
  static fromPathStart(doc: StofDoc, path: string, start: StofNode): StofNode | undefined;
  /**
   * Get the ID of this node reference.
   */
  id(): string;
  /**
   * Invalidate this node with a symbol.
   */
  invalidate(doc: StofDoc, symbol: string): boolean;
  /**
   * Invalidate all on this node.
   */
  invalidateAll(doc: StofDoc): boolean;
  /**
   * Dirty?
   */
  dirty(doc: StofDoc, symbol: string): boolean;
  /**
   * Any dirty symbols?
   */
  anyDirty(doc: StofDoc): boolean;
  /**
   * Validate this node with the symbol.
   */
  validate(doc: StofDoc, symbol: string): boolean;
  /**
   * Validate all for this node.
   */
  validate_all(doc: StofDoc): boolean;
  /**
   * Root node for this reference.
   */
  root(doc: StofDoc): StofNode | undefined;
  /**
   * Exists within the document?
   */
  exists(doc: StofDoc): boolean;
  /**
   * Is a child of the 'parent' node?
   * Returns true if this and parent are equal.
   * Returns true if this node is a granchild or below.
   */
  isChildOf(doc: StofDoc, parent: StofNode): boolean;
  /**
   * Is an immediate child of 'parent'?
   * Will return false if this node is a grandchild or below.
   */
  isImmediateChildOf(doc: StofDoc, parent: StofNode): boolean;
  /**
   * Return the named path of this node.
   * Path is '/' separated and starts at this nodes root.
   */
  path(doc: StofDoc): string;
  /**
   * Return the ID path of this node.
   */
  idPath(doc: StofDoc): (string)[];
  /**
   * Distance to another node in the document.
   */
  distanceTo(doc: StofDoc, other: StofNode): number;
  /**
   * Build this nodes trie for searching through data.
   * Should already be built, but nice to have just in case.
   */
  build_trie(doc: StofDoc): void;
  /**
   * Name of this node.
   */
  name(doc: StofDoc): string | undefined;
  /**
   * Parent of this node.
   */
  parent(doc: StofDoc): StofNode | undefined;
  /**
   * Children of this node.
   */
  children(doc: StofDoc): (StofNode)[];
  /**
   * Data on this node.
   */
  data(doc: StofDoc): (StofData)[];
  /**
   * All data on all children nodes.
   */
  allData(doc: StofDoc): (StofData)[];
  /**
   * Has data?
   */
  hasData(doc: StofDoc, data: StofData): boolean;
  /**
   * Data on this node with an ID that has the prefix 'prefix'.
   */
  prefixData(doc: StofDoc, prefix: string): (StofData)[];
  /**
   * All data on all children nodes with an ID that has the prefix 'prefix'.
   */
  allPrefixData(doc: StofDoc, prefix: string): (StofData)[];
  /**
   * Create some abstract data on this node.
   */
  createData(doc: StofDoc, value: any): StofData;
  /**
   * Create a new field on this node.
   */
  createField(doc: StofDoc, name: string, value: any): StofField;
  /**
   * JSON value of this node as a whole.
   * Can use this to store this value in an external place.
   */
  to_json(doc: StofDoc): any;
  /**
   * Loat a JSON representation of a node into a document.
   * Can use this to load nodes from an external place.
   */
  static from_json(doc: StofDoc, json: any): boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_stofdoc_free: (a: number, b: number) => void;
  readonly stofdoc_new: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
  readonly stofdoc_js: (a: number, b: number, c: any) => [number, number, number];
  readonly stofdoc_bytes: (a: number, b: number, c: any, d: number, e: number) => [number, number, number];
  readonly stofdoc_id: (a: number) => [number, number];
  readonly stofdoc_name: (a: number) => [number, number];
  readonly stofdoc_setName: (a: number, b: number, c: number) => void;
  readonly stofdoc_version: (a: number) => [number, number];
  readonly stofdoc_availableFormats: (a: number) => [number, number];
  readonly stofdoc_formatContentType: (a: number, b: number, c: number) => [number, number];
  readonly stofdoc_headerImport: (a: number, b: number, c: number, d: number, e: number, f: any, g: number, h: number) => [number, number, number];
  readonly stofdoc_stringImport: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
  readonly stofdoc_fileImport: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number];
  readonly stofdoc_exportString: (a: number, b: number, c: number) => [number, number, number, number];
  readonly stofdoc_exportStringFor: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly stofdoc_exportBytes: (a: number, b: number, c: number) => [number, number, number];
  readonly stofdoc_exportBytesFor: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly stofdoc_availableLibraries: (a: number) => [number, number];
  readonly stofdoc_insertLibFunc: (a: number, b: number, c: number, d: number, e: number, f: any) => void;
  readonly stofdoc_createLibs: (a: number) => void;
  readonly stofdoc_get: (a: number, b: number, c: number) => any;
  readonly stofdoc_fieldFromRoot: (a: number, b: number, c: number) => number;
  readonly stofdoc_field: (a: number, b: number, c: number, d: number) => number;
  readonly stofdoc_funcFromRoot: (a: number, b: number, c: number) => number;
  readonly stofdoc_func: (a: number, b: number, c: number, d: number) => number;
  readonly stofdoc_callFunc: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
  readonly stofdoc_run: (a: number) => void;
  readonly stofdoc_runAt: (a: number, b: number) => void;
  readonly stofdoc_mainRoot: (a: number) => number;
  readonly stofdoc_rootByName: (a: number, b: number, c: number) => number;
  readonly stofdoc_isRoot: (a: number, b: number) => number;
  readonly stofdoc_roots: (a: number) => [number, number];
  readonly stofdoc_insertRoot: (a: number, b: number, c: number) => number;
  readonly stofdoc_insertNode: (a: number, b: number, c: number, d: number) => number;
  readonly stofdoc_insertNodeWithId: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly stofdoc_removeNode: (a: number, b: number) => number;
  readonly stofdoc_allChildren: (a: number, b: number) => [number, number];
  readonly stofdoc_putData: (a: number, b: number, c: number) => number;
  readonly stofdoc_removeData: (a: number, b: number) => number;
  readonly stofdoc_removeDataFrom: (a: number, b: number, c: number) => number;
  readonly stofdoc_flush_node_deadpool: (a: number) => [number, number];
  readonly stofdoc_flush_data_deadpool: (a: number) => [number, number];
  readonly stofdoc_flush_nodes: (a: number, b: number) => [number, number];
  readonly stofdoc_flush_data: (a: number, b: number) => [number, number];
  readonly stofnode_fromPath: (a: number, b: number, c: number) => number;
  readonly stofnode_fromPathStart: (a: number, b: number, c: number, d: number) => number;
  readonly stofnode_invalidate: (a: number, b: number, c: number, d: number) => number;
  readonly stofnode_invalidateAll: (a: number, b: number) => number;
  readonly stofnode_dirty: (a: number, b: number, c: number, d: number) => number;
  readonly stofnode_anyDirty: (a: number, b: number) => number;
  readonly stofnode_validate: (a: number, b: number, c: number, d: number) => number;
  readonly stofnode_validate_all: (a: number, b: number) => number;
  readonly stofnode_root: (a: number, b: number) => number;
  readonly stofnode_exists: (a: number, b: number) => number;
  readonly stofnode_isChildOf: (a: number, b: number, c: number) => number;
  readonly stofnode_isImmediateChildOf: (a: number, b: number, c: number) => number;
  readonly stofnode_path: (a: number, b: number) => [number, number];
  readonly stofnode_idPath: (a: number, b: number) => [number, number];
  readonly stofnode_distanceTo: (a: number, b: number, c: number) => number;
  readonly stofnode_build_trie: (a: number, b: number) => void;
  readonly stofnode_name: (a: number, b: number) => [number, number];
  readonly stofnode_parent: (a: number, b: number) => number;
  readonly stofnode_children: (a: number, b: number) => [number, number];
  readonly stofnode_data: (a: number, b: number) => [number, number];
  readonly stofnode_allData: (a: number, b: number) => [number, number];
  readonly stofnode_hasData: (a: number, b: number, c: number) => number;
  readonly stofnode_prefixData: (a: number, b: number, c: number, d: number) => [number, number];
  readonly stofnode_allPrefixData: (a: number, b: number, c: number, d: number) => [number, number];
  readonly stofnode_createData: (a: number, b: number, c: any) => [number, number, number];
  readonly stofnode_createField: (a: number, b: number, c: number, d: number, e: any) => number;
  readonly stofnode_to_json: (a: number, b: number) => any;
  readonly stofnode_from_json: (a: number, b: any) => number;
  readonly __wbg_stofdata_free: (a: number, b: number) => void;
  readonly stofdata_construct: (a: number, b: number, c: any) => [number, number, number];
  readonly stofdata_newWithId: (a: number, b: number, c: number, d: number, e: any) => [number, number, number];
  readonly stofdata_remove: (a: number, b: number) => number;
  readonly stofdata_removeFrom: (a: number, b: number, c: number) => number;
  readonly stofdata_new: (a: number, b: number) => number;
  readonly stofdata_id: (a: number) => [number, number];
  readonly stofdata_invalidate: (a: number, b: number, c: number, d: number) => number;
  readonly stofdata_invalidateValue: (a: number, b: number) => number;
  readonly stofdata_dirty: (a: number, b: number, c: number, d: number) => number;
  readonly stofdata_anyDirty: (a: number, b: number) => number;
  readonly stofdata_validate: (a: number, b: number, c: number, d: number) => number;
  readonly stofdata_validate_all: (a: number, b: number) => number;
  readonly stofdata_exists: (a: number, b: number) => number;
  readonly stofdata_nodes: (a: number, b: number) => [number, number];
  readonly stofdata_getValue: (a: number, b: number) => [number, number, number];
  readonly stofdata_setValue: (a: number, b: number, c: any) => number;
  readonly stofdata_to_json: (a: number, b: number) => any;
  readonly stofdata_from_json: (a: number, b: any) => number;
  readonly __wbg_stoffield_free: (a: number, b: number) => void;
  readonly stoffield_new: (a: number, b: number, c: number, d: any) => number;
  readonly stoffield_fromData: (a: number, b: number) => [number, number, number];
  readonly stoffield_data: (a: number) => number;
  readonly stoffield_name: (a: number) => [number, number];
  readonly stoffield_value: (a: number) => any;
  readonly stoffield_setValue: (a: number, b: number, c: any) => void;
  readonly stoffield_attach: (a: number, b: number, c: number) => void;
  readonly stoffield_remove: (a: number, b: number) => void;
  readonly stoffield_removeFrom: (a: number, b: number, c: number) => void;
  readonly stoffield_fields: (a: number, b: number) => [number, number];
  readonly stoffield_adjacent: (a: number, b: number, c: number, d: number) => number;
  readonly stoffield_fieldFromRoot: (a: number, b: number, c: number) => number;
  readonly stoffield_field: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_stoffunc_free: (a: number, b: number) => void;
  readonly stoffunc_fromData: (a: number, b: number) => [number, number, number];
  readonly stoffunc_data: (a: number) => number;
  readonly stoffunc_name: (a: number) => [number, number];
  readonly stoffunc_returnType: (a: number) => [number, number];
  readonly stoffunc_parameters: (a: number) => [number, number];
  readonly stoffunc_attach: (a: number, b: number, c: number) => void;
  readonly stoffunc_remove: (a: number, b: number) => void;
  readonly stoffunc_removeFrom: (a: number, b: number, c: number) => void;
  readonly stoffunc_funcs: (a: number, b: number) => [number, number];
  readonly stoffunc_adjacent: (a: number, b: number, c: number, d: number) => number;
  readonly stoffunc_funcFromRoot: (a: number, b: number, c: number) => number;
  readonly stoffunc_func: (a: number, b: number, c: number, d: number) => number;
  readonly stoffunc_call: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly __wbg_stoffuncparam_free: (a: number, b: number) => void;
  readonly stoffuncparam_name: (a: number) => [number, number];
  readonly stoffuncparam_type: (a: number) => [number, number];
  readonly stoffuncparam_hasDefault: (a: number) => number;
  readonly stoflib_load: (a: number, b: number) => void;
  readonly stofdoc_createData: (a: number, b: number, c: any) => [number, number, number];
  readonly stofdoc_createDataWithId: (a: number, b: number, c: number, d: number, e: any) => [number, number, number];
  readonly stofnode_id: (a: number) => [number, number];
  readonly stoflib_name: (a: number) => [number, number];
  readonly __wbg_stofnode_free: (a: number, b: number) => void;
  readonly __wbg_stoflib_free: (a: number, b: number) => void;
  readonly stofnode_new: (a: number, b: number) => number;
  readonly stoflib_new: (a: number, b: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_drop_slice: (a: number, b: number) => void;
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
