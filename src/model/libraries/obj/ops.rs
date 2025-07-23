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

use std::sync::Arc;
use imbl::vector;
use crate::{model::{obj::{ANY, AT, ATTRIBUTES, AT_REF, CHILDREN, CONTAINS, CREATE_TYPE, DISTANCE, EMPTY, EXISTS, FIELDS, FUNCS, GET, GET_REF, ID, INSERT, INSTANCE_OF, IS_PARENT, IS_ROOT, LEN, MOVE, MOVE_FIELD, NAME, OBJ_LIB, PARENT, PATH, PROTO, REMOVE, REMOVE_PROTO, ROOT, RUN, SCHEMAFY, SET_PROTO, UPCAST}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, NumT, Type, Val}};


/// Name.
pub fn obj_name() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "name".into(),
        is_async: false,
        docs: "# Name\nReturns the name of an object as a string.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NAME.clone());
            Ok(instructions)
        })
    }
}

/// Id.
pub fn obj_id() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "id".into(),
        is_async: false,
        docs: "# ID\nReturns the id of an object as a string.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ID.clone());
            Ok(instructions)
        })
    }
}

/// Path.
pub fn obj_path() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "path".into(),
        is_async: false,
        docs: "# Path\nReturns the path of an object as a dot separated string of object names.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PATH.clone());
            Ok(instructions)
        })
    }
}

/// Parent.
pub fn obj_parent() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "parent".into(),
        is_async: false,
        docs: "# Parent\nReturns the parent object of a given object or null if the object is a root.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PARENT.clone());
            Ok(instructions)
        })
    }
}

/// Is Parent?
pub fn obj_is_parent() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "is_parent".into(),
        is_async: false,
        docs: "# Is Parent?\nReturns true if this object is a parent of another.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "other".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_PARENT.clone());
            Ok(instructions)
        })
    }
}

/// Exists?
pub fn obj_exists() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "exists".into(),
        is_async: false,
        docs: "# Exists?\nReturns true if this object exists in the graph (objects are just pointers into a graph).".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EXISTS.clone());
            Ok(instructions)
        })
    }
}

/// Children.
pub fn obj_children() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "children".into(),
        is_async: false,
        docs: "# Children\nReturns a list of child objects on this object.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CHILDREN.clone());
            Ok(instructions)
        })
    }
}

/// Root.
pub fn obj_root() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "root".into(),
        is_async: false,
        docs: "# Root\nReturns the root object that contains this object (or self if this object is a root).".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ROOT.clone());
            Ok(instructions)
        })
    }
}

/// Is root?
pub fn obj_is_root() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "is_root".into(),
        is_async: false,
        docs: "# Is Root?\nReturns true if this object is a root object.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_ROOT.clone());
            Ok(instructions)
        })
    }
}

/// Prototype.
pub fn obj_proto() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "prototype".into(),
        is_async: false,
        docs: "# Prototype\nReturns the prototype of this object or null if a prototype doesn't exist.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROTO.clone());
            Ok(instructions)
        })
    }
}

/// Create a type.
pub fn obj_create_type() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "create_type".into(),
        is_async: false,
        docs: "# Create Type\nCreate a type from this object and a type name.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "typename".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CREATE_TYPE.clone());
            Ok(instructions)
        })
    }
}

/// Upcast.
pub fn obj_upcast() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "upcast".into(),
        is_async: false,
        docs: "# Upcast\nSet the prototype of this object to the prototype of this objects prototype.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(UPCAST.clone());
            Ok(instructions)
        })
    }
}

/// Set prototype.
pub fn obj_set_proto() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "set_prototype".into(),
        is_async: false,
        docs: "# Set Prototype\nSet the prototype of this object to either another object or a typename.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "proto".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SET_PROTO.clone());
            Ok(instructions)
        })
    }
}

/// Remove prototype.
pub fn obj_remove_proto() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "remove_prototype".into(),
        is_async: false,
        docs: "# Remove Prototype\nRemove the prototype of this object.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_PROTO.clone());
            Ok(instructions)
        })
    }
}

/// Instance of prototype?
pub fn obj_instance_of_proto() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "instance_of".into(),
        is_async: false,
        docs: "# Is an Instance of a Prototype?\nReturns true if this object is an instance of a prototype.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "proto".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INSTANCE_OF.clone());
            Ok(instructions)
        })
    }
}

/// Length.
pub fn obj_len() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: "# Length (Number of Fields)\nReturns the number of fields on this object.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN.clone());
            Ok(instructions)
        })
    }
}

/// At.
pub fn obj_at() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: "# At (index operator)\nReturns the field (tuple of name and value) on this object at the given index or null if out of bounds.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(AT_REF.clone());
            } else {
                instructions.push(AT.clone());
            }
            Ok(instructions)
        })
    }
}

/// Get.
pub fn obj_get() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "get".into(),
        is_async: false,
        docs: "# Get\nReturns data on this object by name (field, fn, or opaque data reference).".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "name".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(GET_REF.clone());
            } else {
                instructions.push(GET.clone());
            }
            Ok(instructions)
        })
    }
}

/// Contains?
pub fn obj_contains() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "contains".into(),
        is_async: false,
        docs: "# Contains?\nReturns true if this object contains some data with the given name.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "name".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CONTAINS.clone());
            Ok(instructions)
        })
    }
}

/// Insert.
pub fn obj_insert() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "insert".into(),
        is_async: false,
        docs: "# Insert\nPerforms a 'set variable' instruction just like a normal field assignment, using this object as a starting context.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "field_path".into(), param_type: Type::Str, default: None },
            Param { name: "value".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INSERT.clone());
            Ok(instructions)
        })
    }
}

/// Remove.
pub fn obj_remove() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "remove".into(),
        is_async: false,
        docs: "# Remove\nPerforms a 'drop' just like the Std function, using this object as a starting context.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "field_path".into(), param_type: Type::Str, default: None },
            Param { name: "shallow".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE.clone());
            Ok(instructions)
        })
    }
}

/// Move field.
pub fn obj_move_field() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "move_field".into(),
        is_async: false,
        docs: "# Move (or Rename) Field\nMoves a field from a source path/name to a destination path/name. Returns true if successful.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "source".into(), param_type: Type::Str, default: None },
            Param { name: "dest".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MOVE_FIELD.clone());
            Ok(instructions)
        })
    }
}

/// Fields.
pub fn obj_fields() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "fields".into(),
        is_async: false,
        docs: "# Fields\nReturns a list of fields (tuple with name and value) on this object.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FIELDS.clone());
            Ok(instructions)
        })
    }
}

/// Funcs.
pub fn obj_funcs() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "funcs".into(),
        is_async: false,
        docs: "# Functions\nReturns a list of functions on this object, optionally filtering by attributes (string or list/tuple/set of strings).".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "attributes".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FUNCS.clone());
            Ok(instructions)
        })
    }
}

/// Empty?
pub fn obj_empty() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "empty".into(),
        is_async: false,
        docs: "# Empty?\nReturns true if this object doesn't have any data (fields, funcs, etc.)".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EMPTY.clone());
            Ok(instructions)
        })
    }
}

/// Any?
pub fn obj_any() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "any".into(),
        is_async: false,
        docs: "# Any?\nReturns true if this object has at least one datum (fields, funcs, etc.)".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ANY.clone());
            Ok(instructions)
        })
    }
}

/// Attributes.
pub fn obj_attributes() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "attributes".into(),
        is_async: false,
        docs: "# Attributes.\nReturns a map of attributes, either for this node or a field/func/obj at a given string path.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "path".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ATTRIBUTES.clone());
            Ok(instructions)
        })
    }
}

/// Move.
pub fn obj_move() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "move".into(),
        is_async: false,
        docs: "# Move Object.\nMove this object to a new parent. Destination/new parent must not be a child of this node (will return false and not be moved). Returns true if successfully moved.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "dest".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MOVE.clone());
            Ok(instructions)
        })
    }
}

/// Distance.
pub fn obj_dist() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "dist".into(),
        is_async: false,
        docs: "# Object Distance.\nReturns the distance between two objects in the same graph.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "other".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DISTANCE.clone());
            Ok(instructions)
        })
    }
}

/// Run.
pub fn obj_run() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "run".into(),
        is_async: false,
        docs: "# Run Object\nCalls all #[run] functions with an optional order on this object, also going into fields, running sub-objects, etc.".into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(RUN.clone());
            Ok(instructions)
        })
    }
}

/// Schemafy.
pub fn obj_schemafy() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "schemafy".into(),
        is_async: false,
        docs: "# Schemafy\nApplies all #[schema] fields from a schema object onto a target object, returning true if the target is determined to be valid according to the schema.".into(),
        params: vector![
            Param { name: "schema".into(), param_type: Type::Void, default: None },
            Param { name: "target".into(), param_type: Type::Void, default: None },
            Param { name: "remove_invalid".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
            Param { name: "remove_undefined".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SCHEMAFY.clone());
            Ok(instructions)
        })
    }
}
