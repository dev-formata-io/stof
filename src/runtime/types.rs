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

use std::ops::Deref;
use arcstr::{literal, ArcStr};
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, NodeRef, SId, PROTOTYPE_TYPE_ATTR, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, parser::types::parse_type_complete, runtime::{Units, Val}};


// Literal string types.
const NULL: ArcStr = literal!("null");
const VOID: ArcStr = literal!("void");
const UNKNOWN: ArcStr = literal!("unknown");
const MAP: ArcStr = literal!("map");
const SET: ArcStr = literal!("set");
const LIST: ArcStr = literal!("list");
const BOOL: ArcStr = literal!("bool");
const BLOB: ArcStr = literal!("blob");
const FUNC: ArcStr = literal!("fn");
pub(super) const DATA: ArcStr = literal!("data");
pub(super) const OBJ: ArcStr = literal!("obj");
const VER: ArcStr = literal!("ver");
const STR: ArcStr = literal!("str");
const PROMPT: ArcStr = literal!("prompt");
const INT: ArcStr = literal!("int");
const FLOAT: ArcStr = literal!("float");


#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash)]
/// Type.
pub enum Type {
    #[default]
    Void,
    Null,

    // Type that does not match null (used with func args that have a !)
    NotNull(Box<Self>),

    Promise(Box<Self>),

    Bool,
    Num(NumT),
    Str,
    Ver,
    Prompt,

    Obj(SId), // Prototypes
    Fn,
    Data(ArcStr), // typetag lib linking, etc.

    Blob,

    List,
    Tup(Vector<Self>),
    Map,
    Set,

    Unknown,
    Union(Vector<Self>),

    /// A function with a signature: `fn(str, int) -> str` (parameter types, then the return type).
    /// Matches a compatible function (see `Type::fits_fn`); a missing return type is Unknown (any return).
    /// Last on purpose: BSTF stores a type by its position here, so new types go at the end (older documents keep
    /// loading the same).
    FnSig(Vector<Self>, Box<Self>),
}
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        // not-null: the same type without null. Two not-null types compare their types; a not-null type and a plain
        // one compare the types (a value's own type is never not-null); null never matches a not-null type.
        match (self, other) {
            (Self::NotNull(a), Self::NotNull(b)) => return a.deref() == b.deref(),
            (Self::NotNull(_), Self::Null) | (Self::Null, Self::NotNull(_)) => return false,
            (Self::NotNull(a), b) => return a.deref() == b,
            (a, Self::NotNull(b)) => return a == b.deref(),
            _ => {}
        }
        match other {
            Self::Null => return match self {
                Self::Union(types) => types.iter().any(|ty| ty == &Self::Null), // null if any of its types can be
                _ => !self.exp_not_null(),
            },
            Self::Unknown => return true,
            Self::Union(types) => {
                match self {
                    Self::Union(otypes) => {
                        for ty in types {
                            for oty in otypes {
                                if ty.eq(oty) {
                                    return true;
                                }
                            }
                        }
                        return false;
                    },
                    sf => {
                        for ty in types {
                            if ty.eq(sf) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
            },
            Self::Promise(ty) => {
                match self {
                    Self::Promise(oty) => return ty == oty,
                    _ => return **ty == *self,
                }
            },
            _ => {}
        }
        match self {
            Self::Union(types) => {
                match other {
                    Self::Union(otypes) => {
                        for ty in types {
                            for oty in otypes {
                                if ty.eq(oty) {
                                    return true;
                                }
                            }
                        }
                        return false;
                    },
                    other => {
                        for ty in types {
                            if ty.eq(other) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
            },
            Self::Void => {
                match other {
                    Self::Void => true,
                    _ => false,
                }
            },
            Self::Bool => {
                match other {
                    Self::Bool => true,
                    _ => false,
                }
            },
            Self::Num(t) => {
                match other {
                    Self::Num(ot) => t.eq(ot),
                    _ => false,
                }
            },
            Self::Prompt => {
                match other {
                    Self::Prompt => true,
                    _ =>  false,
                }
            },
            Self::Str => {
                match other {
                    Self::Str => true,
                    _ => false,
                }
            },
            Self::Ver => {
                match other {
                    Self::Ver => true,
                    _ => false,
                }
            },
            Self::Obj(t) => {
                match other {
                    Self::Obj(ot) => t.eq(ot),
                    _ => false,
                }
            },
            Self::Fn => {
                match other {
                    Self::Fn => true,
                    _ => false,
                }
            },
            Self::FnSig(params, ret) => {
                match other {
                    Self::FnSig(oparams, oret) => params.len() == oparams.len() && params.iter().zip(oparams.iter()).all(|(a, b)| a == b) && ret == oret,
                    _ => false, // a function value is checked against its signature in cast (Val::fits_fn_sig)
                }
            },
            Self::Data(t) => {
                match other {
                    Self::Data(ot) => t.eq(ot),
                    _ => false, // gen type handled in cast
                }
            },
            Self::List => {
                match other {
                    Self::List => true,
                    _ => false,
                }
            },
            Self::Map => {
                match other {
                    Self::Map => true,
                    _ => false,
                }
            },
            Self::Set => {
                match other {
                    Self::Set => true,
                    _ => false,
                }
            },
            Self::Tup(types) => {
                match other {
                    Self::Tup(otypes) => types.eq(otypes),
                    _ => false,
                }
            },
            Self::Blob => {
                match other {
                    Self::Blob => true,
                    _ => false,
                }
            },
            Self::Promise(ty) => {
                match other {
                    Self::Promise(oty) => ty == oty,
                    _ => **ty == *other,
                }
            },
            Self::Null => !other.exp_not_null(),
            Self::NotNull(t) => t.deref().eq(other),
            Self::Unknown => true,
        }
    }
}
impl Eq for Type {}
impl Type {
    /// Does a function fit a signature type (`fn(P..) -> R`)? A compatible match, not an exact one:
    /// - it can be called with the signature's arguments: as many as the signature has, and any parameters past
    ///   those have defaults (Ex. `fn(str)` fits `fn(name: str, greeting: str = 'hi')`);
    /// - each parameter accepts the signature's type for it (the same type, a wider one such as `unknown`, a union
    ///   that includes it, a number for a number, or a prototype the signature's type extends);
    /// - its return type fits the signature's (the same, or narrower), unless the signature leaves it out (any return).
    ///   An async function fits by what it returns (no `Promise<..>` in the signature).
    ///
    /// `func_params` are the function's parameter types, each with whether it has a default. Object type names in both
    /// should already be prototypes (`obj_to_proto`), so a prototype that extends another is seen.
    pub fn fits_fn(params: &Vector<Type>, ret: &Type, func_params: &[(Type, bool)], func_ret: &Type, graph: &Graph) -> bool {
        if params.len() > func_params.len() { return false; }
        if func_params.iter().skip(params.len()).any(|(_, has_default)| !has_default) { return false; }
        for (expected, (param_type, _)) in params.iter().zip(func_params.iter()) {
            if !Self::accepts(param_type, expected, graph) { return false; }
        }
        match ret {
            Self::Unknown => true,
            ret => Self::accepts(ret, func_ret, graph),
        }
    }

    /// Can a value of type `given` go where `declared` is expected (for a function signature)?
    fn accepts(declared: &Type, given: &Type, graph: &Graph) -> bool {
        match (declared, given) {
            // a not-null parameter needs a not-null argument (a plain type can be null); a not-null value goes
            // anywhere its type does
            (Self::NotNull(declared), Self::NotNull(given)) => Self::accepts(declared, given, graph),
            (Self::NotNull(_), _) => false,
            (declared, Self::NotNull(given)) => Self::accepts(declared, given, graph),
            (Self::Num(NumT::Int | NumT::Float), Self::Num(NumT::Int | NumT::Float)) => true,
            (Self::Obj(declared), Self::Obj(given)) => {
                if declared == given || declared.as_ref() == OBJ.as_str() { return true; }
                given.node_exists(graph) && declared.node_exists(graph) &&
                    Val::Obj(given.clone()).instance_of(declared, graph).unwrap_or(false)
            },
            (Self::FnSig(dparams, dret), Self::FnSig(gparams, gret)) => {
                // a function parameter: what's passed must work everywhere the declared one would be called
                dparams.len() == gparams.len() &&
                    dparams.iter().zip(gparams.iter()).all(|(d, g)| Self::accepts(g, d, graph)) &&
                    (matches!(dret.as_ref(), Self::Unknown) || Self::accepts(dret, gret, graph))
            },
            (Self::Fn, Self::FnSig(..)) => true,
            (Self::Union(types), given) => types.iter().any(|ty| Self::accepts(ty, given, graph)),
            (declared, Self::Union(types)) if !matches!(declared, Self::Unknown) => types.iter().all(|ty| Self::accepts(declared, ty, graph)),
            (declared, given) => declared == given,
        }
    }

    #[inline]
    pub fn empty(&self) -> bool {
        match self {
            Self::Null |
            Self::Void => true,
            _ => false,
        }
    }

    #[inline]
    pub fn null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }

    #[inline]
    pub fn exp_not_null(&self) -> bool {
        match self {
            Self::NotNull(_) => true,
            _ => false,
        }
    }

    /// If this is an object type as a name, find the prototype and swap it.
    /// Or if a union, make sure all objects in the union are NodeRefs.
    pub fn obj_to_proto(&mut self, graph: &Graph, mut context: Option<NodeRef>) {
        match &mut *self {
            Self::Union(types) => {
                for ty in types { ty.obj_to_proto(graph, context.clone()); }
            },
            Self::Promise(ctype) => {
                ctype.obj_to_proto(graph, context);
            },
            Self::NotNull(ctype) => {
                ctype.obj_to_proto(graph, context);
            },
            Self::Tup(types) => {
                for ty in types { ty.obj_to_proto(graph, context.clone()); }
            },
            Self::FnSig(params, ret) => {
                for ty in params { ty.obj_to_proto(graph, context.clone()); }
                ret.obj_to_proto(graph, context);
            },
            Self::Obj(name_or_id) => {
                if !name_or_id.node_exists(graph) { // not a node ref, so must be a name
                    let mut path = name_or_id.as_ref().to_string();
                    if path.contains('.') {
                        let mut split = path.split('.').collect::<Vec<_>>();
                        let typename = split.pop().unwrap();
                        let qualifier = split.join(".");

                        let relative = path.starts_with(SELF_STR_KEYWORD.as_str()) || path.starts_with(SUPER_STR_KEYWORD.as_str());
                        if !relative {
                            // Qualified type name (Ex. <Geometry.Point>): the nearest type named "Point" whose path ends with "Geometry"
                            if let Some(proto_id) = graph.find_qualified_type(&qualifier, typename, context.clone()) {
                                *name_or_id = proto_id;
                                return;
                            }
                            context = None; // otherwise an absolute path (starts at a graph root)
                        }

                        // Path to an object, then the nearest type from there (Ex. <self.Point>, <root.a.ref.Point>)
                        if let Some(node) = graph.find_node_named(&qualifier, context.clone()) {
                            context = Some(node);
                            path = typename.to_string();
                        }
                    }

                    if context.is_none() { context = graph.main_root(); }
                    if let Some(proto_id) = graph.find_type(&path, context) {
                        *name_or_id = proto_id;
                    }
                }
            },
            _ => {}
        }
    }

    pub fn type_of(&self) -> ArcStr {
        match self {
            Self::Union(types) => {
                let mut geo = String::default();
                for ty in types {
                    // a function with a return type in a union is in parentheses (its return type would take the rest)
                    let text = match ty {
                        Self::FnSig(_, ret) if !matches!(ret.as_ref(), Self::Unknown) => format!("({})", ty.type_of()),
                        _ => ty.type_of().to_string(),
                    };
                    if geo.len() < 1 {
                        geo.push_str(&text);
                    } else {
                        geo.push_str(&format!(" | {}", text));
                    }
                }
                geo.into()
            },
            Self::Unknown => UNKNOWN,
            Self::Map => MAP,
            Self::Set => SET,
            Self::List => LIST,
            Self::Bool => BOOL,
            Self::Blob => BLOB,
            Self::Fn => FUNC,
            Self::FnSig(params, ret) => {
                let params = params.iter().map(|ty| ty.type_of().to_string()).collect::<Vec<_>>().join(", ");
                match ret.as_ref() {
                    Self::Unknown => format!("fn({params})").into(),
                    ret => format!("fn({params}) -> {}", ret.type_of()).into(),
                }
            },
            Self::Data(tname) => {
                let dta = DATA;
                if tname == &dta {
                    return dta;
                }
                format!("Data<{}>", tname).into()
            },
            Self::Null => NULL,
            Self::NotNull(t) => match t.as_ref() {
                Self::Union(_) => format!("({})!", t.type_of()).into(),
                Self::NotNull(_) => t.type_of(),
                t => format!("{}!", t.type_of()).into(),
            },
            Self::Num(num) => num.type_of(),
            Self::Ver => VER,
            Self::Str => STR,
            Self::Prompt => PROMPT,
            Self::Tup(vals) => {
                let mut res = "(".to_string();
                for i in 0..vals.len() {
                    let v = &vals[i];
                    let type_of = v.type_of();
                    if i < vals.len() - 1 {
                        res.push_str(&format!("{}, ", type_of));
                    } else {
                        res.push_str(&type_of);
                    }
                }
                res.push_str(")");
                res.into()
            },
            Self::Void => VOID,
            Self::Obj(ctype) => ctype.as_ref().into(),
            Self::Promise(ty) => format!("Promise<{}>", ty.type_of()).into(),
        }
    }

    pub fn rt_type_of(&self, graph: &Graph) -> ArcStr {
        match self {
            Self::Union(types) => {
                let mut geo = String::default();
                for ty in types {
                    // a function with a return type in a union is in parentheses (its return type would take the rest)
                    let text = match ty {
                        Self::FnSig(_, ret) if !matches!(ret.as_ref(), Self::Unknown) => format!("({})", ty.rt_type_of(graph)),
                        _ => ty.rt_type_of(graph).to_string(),
                    };
                    if geo.len() < 1 {
                        geo.push_str(&text);
                    } else {
                        geo.push_str(&format!(" | {}", text));
                    }
                }
                geo.into()
            },
            Self::Unknown => UNKNOWN,
            Self::Map => MAP,
            Self::Set => SET,
            Self::List => LIST,
            Self::Bool => BOOL,
            Self::Blob => BLOB,
            Self::Fn => FUNC,
            Self::FnSig(params, ret) => {
                let params = params.iter().map(|ty| ty.rt_type_of(graph).to_string()).collect::<Vec<_>>().join(", ");
                match ret.as_ref() {
                    Self::Unknown => format!("fn({params})").into(),
                    ret => format!("fn({params}) -> {}", ret.rt_type_of(graph)).into(),
                }
            },
            Self::Data(tname) => {
                let dta = DATA;
                if tname == &dta {
                    return dta;
                }
                format!("Data<{}>", tname).into()
            },
            Self::Null => NULL,
            Self::NotNull(t) => match t.as_ref() {
                Self::Union(_) => format!("({})!", t.rt_type_of(graph)).into(),
                Self::NotNull(_) => t.rt_type_of(graph),
                t => format!("{}!", t.rt_type_of(graph)).into(),
            },
            Self::Num(num) => num.type_of(),
            Self::Ver => VER,
            Self::Str => STR,
            Self::Prompt => PROMPT,
            Self::Tup(vals) => {
                let mut res = "(".to_string();
                for i in 0..vals.len() {
                    let v = &vals[i];
                    let type_of = v.rt_type_of(graph);
                    if i < vals.len() - 1 {
                        res.push_str(&format!("{}, ", type_of));
                    } else {
                        res.push_str(&type_of);
                    }
                }
                res.push_str(")");
                res.into()
            },
            Self::Void => VOID,
            Self::Obj(ctype) => {
                if let Some(node) = ctype.node(graph) {
                    if let Some(type_attr) = node.attributes.get(PROTOTYPE_TYPE_ATTR.as_str()) {
                        match type_attr {
                            Val::Str(name) => {
                                return name.clone();
                            },
                            _ => {
                                return node.name.as_ref().into();
                            }
                        }
                    }
                }
                ctype.as_ref().into()
            },
            Self::Promise(ty) => format!("Promise<{}>", ty.rt_type_of(graph)).into(),
        }
    }

    pub fn md_type_of(&self, graph: &Graph) -> String {
        self.rt_type_of(graph).replace("<", "\\<")
    }

    /// Generic libname.
    pub fn gen_lib_name(&self) -> ArcStr {
        match self {
            Self::Unknown |
            Self::Null |
            Self::Union(_) |
            Self::Void => literal!("Empty"),
            Self::List => literal!("List"),
            Self::Map => literal!("Map"),
            Self::Set => literal!("Set"),
            Self::Blob => literal!("Blob"),
            Self::Bool => literal!("Bool"),
            Self::Fn |
            Self::FnSig(..) => literal!("Fn"),
            Self::Num(_) => literal!("Num"),
            Self::Data(_) => literal!("Data"),
            Self::Str => literal!("Str"),
            Self::Prompt => literal!("Prompt"),
            Self::Obj(_) => literal!("Obj"),
            Self::Promise(_) => literal!("Promise"),
            Self::Ver => literal!("Ver"),
            Self::Tup(_) => literal!("Tup"),
            Self::NotNull(t) => t.gen_lib_name(),
        }
    }
}
impl<T: AsRef<str>> From<T> for Type {
    fn from(value: T) -> Self {
        // a name that isn't a type expression is an object type's name (it won't resolve to a prototype)
        parse_type_complete(value.as_ref()).unwrap_or_else(|_| Type::Obj(value.as_ref().into()))
    }
}
impl ToString for Type {
    fn to_string(&self) -> String {
        self.type_of().to_string()
    }
}


#[derive(Debug, Clone, Copy, Deserialize, Serialize, Hash)]
/// Number Type.
pub enum NumT {
    Int,
    Float,
    Units(Units),
}
impl PartialEq for NumT {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Int => {
                match other {
                    Self::Int => true,
                    _ => false,
                }
            },
            Self::Float => {
                match other {
                    Self::Float => true,
                    _ => false,
                }
            },
            Self::Units(units) => {
                match other {
                    Self::Float => true,
                    Self::Units(ounits) => {
                        units == ounits
                    },
                    _ => false,
                }
            },
        }
    }
}
impl Eq for NumT {}
impl NumT {
    pub fn type_of(&self) -> ArcStr {
        match self {
            Self::Float => FLOAT,
            Self::Int => INT,
            Self::Units(units) => units.to_string(),
        }
    }
}

#[cfg(test)]
mod not_null_tests {
    use super::*;
    use imbl::vector;

    fn nn(ty: Type) -> Type { Type::NotNull(Box::new(ty)) }

    #[test]
    fn not_null_equality() {
        let int = Type::Num(NumT::Int);
        // two not-null types compare their types
        assert!(nn(int.clone()) == nn(int.clone()));
        assert!(nn(int.clone()) != nn(Type::Str));
        assert!(nn(nn(int.clone())) == nn(int.clone()));
        // a not-null type and a plain one compare the types, either way around
        assert!(nn(int.clone()) == int.clone());
        assert!(int.clone() == nn(int.clone()));
        assert!(nn(int.clone()) != Type::Str);
        assert!(Type::Str != nn(int.clone()));
        // null never matches a not-null type, either way around (even not-null unknown)
        assert!(nn(int.clone()) != Type::Null);
        assert!(Type::Null != nn(int.clone()));
        assert!(nn(Type::Unknown) != Type::Null);
        assert!(Type::Null != nn(Type::Unknown));
        // unknown still matches the type
        assert!(Type::Unknown == nn(int.clone()));
        assert!(nn(int.clone()) == Type::Unknown);
        assert!(nn(Type::Unknown) == Type::Str);
        // plain types still take null
        assert!(int.clone() == Type::Null);
        assert!(Type::Null == int.clone());
        // inside unions, tuples, promises, and signatures
        let union = Type::Union(vector![nn(int.clone()), Type::Str]);
        assert!(union == int.clone());
        assert!(union == Type::Null); // str can be null
        assert!(Type::Union(vector![nn(int.clone()), nn(Type::Str)]) != Type::Null);
        assert!(Type::Null != Type::Union(vector![nn(int.clone()), nn(Type::Str)]));
        assert!(nn(Type::Union(vector![int.clone(), Type::Str])) != Type::Null);
        assert!(nn(Type::Union(vector![int.clone(), Type::Str])) == Type::Str);
        assert!(Type::Tup(vector![nn(int.clone()), Type::Str]) == Type::Tup(vector![nn(int.clone()), Type::Str]));
        assert!(Type::Tup(vector![nn(int.clone()), Type::Str]) == Type::Tup(vector![int.clone(), Type::Str]));
        assert!(Type::Tup(vector![nn(int.clone())]) != Type::Tup(vector![Type::Null]));
        assert!(Type::Promise(Box::new(nn(int.clone()))) == Type::Promise(Box::new(nn(int.clone()))));
        let sig = Type::FnSig(vector![nn(int.clone())], Box::new(nn(Type::Str)));
        assert!(sig == sig.clone());
    }
}

#[cfg(test)]
mod bstf_tests {
    use super::*;

    #[test]
    /// BSTF (bincode) stores a type by its position in the enum. Moving a variant changes what every stored
    /// document's types mean (Ex. a `list` read back as a `blob`), so new types go at the end.
    fn type_positions_are_stable() {
        let position = |ty: Type| u32::from_le_bytes(bincode::serialize(&ty).unwrap()[..4].try_into().unwrap());
        assert_eq!(position(Type::Void), 0);
        assert_eq!(position(Type::Null), 1);
        assert_eq!(position(Type::Bool), 4);
        assert_eq!(position(Type::Str), 6);
        assert_eq!(position(Type::Fn), 10);
        assert_eq!(position(Type::Blob), 12);
        assert_eq!(position(Type::List), 13);
        assert_eq!(position(Type::Map), 15);
        assert_eq!(position(Type::Set), 16);
        assert_eq!(position(Type::Unknown), 17);
        assert_eq!(position(Type::Union(Vector::default())), 18);
        assert_eq!(position(Type::FnSig(Vector::default(), Box::new(Type::Unknown))), 19);
    }
}
