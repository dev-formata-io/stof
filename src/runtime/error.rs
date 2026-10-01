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

use std::fmt::Display;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use crate::{parser::doc::StofParseError, runtime::{Type, Val}};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Error.
pub enum Error {

    /*****************************************************************************
     * Format Errors.
     *****************************************************************************/
    FormatStringImportNotImplemented(String),
    FormatFileImportFsError(String),
    FormatFileExportFsError(String),
    FormatFileImportNotAllowed,
    FormatFileExportNotAllowed,
    FormatBinaryImportUtf8Error,
    FormatStringExportNotImplemented(String),
    GraphFormatNotFound,
    RelativeImportWithoutContext,
    ImportOsStringError,

    JSONStringImport(String),
    JSONStringExport(String),

    TOMLStringImport(String),
    TOMLStringExport(String),

    YAMLStringImport(String),
    YAMLStringExport(String),

    BYTESExport(String),

    PKGImport(String),

    BSTFImport(String),
    BSTFExport(String),

    PDFImport(String),
    PDFExport(String),

    ImageImport(String),
    ImageExport(String),

    DocXImport(String),

    /*****************************************************************************
     * Filesystem Library.
     *****************************************************************************/
    FsReadStringStackError,
    FsReadStringError(String),
    FsReadStackError,
    FsReadError(String),
    FsWriteStackError,
    FsWriteError(String),

    /*****************************************************************************
     * Standard Library Errors.
     *****************************************************************************/
    Thrown(Val),
    AssertFailed(String),
    AssertNotFailed(String),
    AssertEqFailed(String),
    AssertNotEqFailed(String),
    MapConstructor(String),
    StdFunctions,
    StdParse(String),
    StdBlobify(String),
    StdStringify(String),
    StdHasFormat(String),
    StdHasLib(String),
    StdFormatContentType(String),
    StdEnv,
    StdSetEnv,
    StdRemoveEnv,

    /*****************************************************************************
     * HTTP Lib Errors.
     *****************************************************************************/
    HttpArgs(String),
    HttpSendError(String),

    /*****************************************************************************
     * Time Lib Errors.
     *****************************************************************************/
    TimeDiff,
    TimeDiffNano,
    TimeSleep,
    TimeToRFC3339,
    TimeToRFC2822,
    TimeFromRFC3339,
    TimeFromRFC2822,

    /*****************************************************************************
     * Func Lib Errors.
     *****************************************************************************/
    FnId,
    FnData,
    FnBind,
    FnName,
    FnParams,
    FnReturnType,
    FnHasAttr,
    FnAttributes,
    FnObj,
    FnObjs,
    FnIsAsync,
    FnCall,
    FnExpandCall,

    /*****************************************************************************
     * Semver Lib Errors.
     *****************************************************************************/
    VerMajor,
    VerSetMajor,
    VerMinor,
    VerSetMinor,
    VerPatch,
    VerSetPatch,
    VerRelease,
    VerSetRelease,
    VerClearRelease,
    VerBuild,
    VerSetBuild,
    VerClearBuild,

    /*****************************************************************************
     * String Lib Errors.
     *****************************************************************************/
    StrLen,
    StrAt,
    StrFirst,
    StrLast,
    StrStartsWith,
    StrEndsWith,
    StrPush,
    StrContains,
    StrIndexOf,
    StrReplace,
    StrSplit,
    StrUpper,
    StrLower,
    StrTrim,
    StrTrimStart,
    StrTrimEnd,
    StrSubstring,
    StrRegexFail,
    StrIsMatch,
    StrFindAll,

    /*****************************************************************************
     * Prompt Lib Errors.
     *****************************************************************************/
    PromptStr,
    PromptText,
    PromptTag,
    PromptPrompts,
    PromptSetText,
    PromptSetTag,
    PromptLen,
    PromptAt,
    PromptEmpty,
    PromptAny,
    PromptPush,
    PromptPop,
    PromptClear,
    PromptReverse,
    PromptRemove,
    PromptInsert,
    PromptReplace,

    /*****************************************************************************
     * Number Lib Errors.
     *****************************************************************************/
    NumAbs,
    NumSqrt,
    NumCbrt,
    NumFloor,
    NumCeil,
    NumTrunc,
    NumFract,
    NumSignum,
    NumExp,
    NumExp2,
    NumLn,
    NumAt,
    NumRound,
    NumRound2,
    NumPow,
    NumLog,
    NumATan2,
    NumNan,
    NumInf,

    NumHasUnits,
    NumToUnits,
    NumIsAngle,
    NumIsTemp,
    NumIsLength,
    NumIsTime,
    NumIsMass,
    NumIsMemory,
    NumRemoveUnits,

    NumSin,
    NumCos,
    NumTan,
    NumASin,
    NumACos,
    NumATan,
    NumSinH,
    NumCosH,
    NumTanH,
    NumASinH,
    NumACosH,
    NumATanH,

    NumHex,
    NumBin,
    NumOct,
    NumStr,

    /*****************************************************************************
     * Map Lib Errors.
     *****************************************************************************/
    MapAppendOther,
    MapClear,
    MapContains,
    MapFirst,
    MapLast,
    MapGet,
    MapInsert,
    MapEmpty,
    MapAny,
    MapKeys,
    MapValues,
    MapLen,
    MapAt,
    MapPopFirst,
    MapPopLast,
    MapRemove,

    /*****************************************************************************
     * Set Lib Errors.
     *****************************************************************************/
    SetAppendOther,
    SetClear,
    SetContains,
    SetFirst,
    SetLast,
    SetInsert,
    SetSplit,
    SetEmpty,
    SetAny,
    SetLen,
    SetAt,
    SetPopFirst,
    SetPopLast,
    SetRemove,
    SetUnion,
    SetDifference,
    SetIntersection,
    SetSymmetricDifference,
    SetDisjoint,
    SetSubset,
    SetSuperset,
    SetIsUniform,
    SetToUniform,

    /*****************************************************************************
     * List Lib Errors.
     *****************************************************************************/
    ListAppendOther,
    ListPushBack,
    ListPushFront,
    ListPopFront,
    ListPopBack,
    ListClear,
    ListReverse,
    ListReversed,
    ListLen,
    ListAt,
    ListEmpty,
    ListAny,
    ListFirst,
    ListLast,
    ListJoin,
    ListContains,
    ListIndexOf,
    ListRemove,
    ListRemoveFirst,
    ListRemoveLast,
    ListRemoveAll,
    ListInsert,
    ListReplace,
    ListSort,
    ListSortBy,
    ListIsUniform,
    ListToUniform,

    /*****************************************************************************
     * Data Lib Errors.
     *****************************************************************************/
    DataId,
    DataTagname,
    DataExists,
    DataObjs,
    DataDrop,
    DataAttach,
    DataMove,
    DataField,
    DataFromId,
    DataToBlob,
    DataFromBlob,
    DataInvalidate,
    DataValidate,

    PdfExtractImages,
    PdfExtractText,

    ImageWidth,
    ImageHeight,
    ImageGrayscale,
    ImageInvert,
    ImageFlipVertical,
    ImageFlipHorizontal,
    ImageRotate90,
    ImageRotate180,
    ImageRotate270,
    ImageResize,
    ImageResizeExact,
    ImageThumbnail,
    ImageThumbnailExact,
    ImageBlur,
    ImageBlurFast,
    ImageAdjustContrast,
    ImageBrighten,
    ImageBlob,
    ImagePng,
    ImageJpeg,
    ImageGif,
    ImageWebp,
    ImageTiff,
    ImageBmp,
    ImageIco,
    ImageFromBlob,

    AgeNoMatchingKeys,

    /*****************************************************************************
     * Tuple Lib Errors.
     *****************************************************************************/
    TupLen,
    TupAt,

    /*****************************************************************************
     * Blob Lib Errors.
     *****************************************************************************/
    BlobLen,
    BlobAt,
    BlobUtf8Str,
    BlobBase64Str,
    BlobUrlSafeBase64Str,
    BlobFromUtf8Str,
    BlobFromBase64Str,
    BlobFromUrlSafeBase64Str,

    /*****************************************************************************
     * Object Lib Errors.
     *****************************************************************************/
    ObjName,
    ObjId,
    ObjPath,
    ObjParent,
    ObjIsParent,
    ObjExists,
    ObjChildren,
    ObjRoot,
    ObjIsRoot,

    ObjProto,
    ObjSetProto,
    ObjRemoveProto,
    ObjInstanceOf,
    ObjUpcast,
    ObjCreateType,

    ObjLen,
    ObjAt,
    ObjAtRef,
    ObjGet,
    ObjGetRef,
    ObjContains,
    ObjInsert,
    ObjRemove,
    ObjMoveField,
    ObjFields,
    ObjFuncs,
    ObjEmpty,
    ObjAny,
    ObjAttributes,
    ObjMove,
    ObjDistance,
    ObjRun,
    ObjSchemafy,
    ObjDiff,
    ObjToMap,
    ObjToMapRef,
    ObjFromMap,
    ObjFromId,

    ObjNewStack,

    /*****************************************************************************
     * Cast Errors.
     *****************************************************************************/
    ObjectCastProtoDne,

    /*****************************************************************************
     * Await Errors.
     *****************************************************************************/
    AwaitError(Box<Self>),

    /*****************************************************************************
     * Parse Errors.
     *****************************************************************************/
    ParseError(StofParseError),

    /*****************************************************************************
     * Old.
     *****************************************************************************/
    Custom(ArcStr),
    NotImplemented,

    DeclareExisting,
    DeclareInvalidName,
    AssignConst,
    VariableSet,
    FieldReadOnlySet,
    AssignSelf,
    AssignSuper,
    AssignRootNonObj,
    AssignExistingRoot,

    StackError,
    ExecutionTimeout,
    StackOverflow,
    SelfStackError,
    NewStackError,
    CallStackError,
    CallStackOverflow,
    CastStackError,
    CastVal(Type, Type),

    // Function calling errors
    FuncDne(String),
    FuncDefaultArg(Box<Self>),
    FuncArgs,
    FuncInvalidReturn,

    // Value errors
    Truthy,
    IsNull,
    NotTruthy,
    GreaterThan,
    GreaterOrEq,
    LessThan,
    LessOrEq,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    AND,
    OR,
    XOR,
    SHL,
    SHR,

    /// An error returned to a host (Runtime::call), with the Stof call stack where it happened.
    /// Added at the end for rev-compatibility.
    Located(Box<Self>, String),

    /// Arithmetic (+ - * / %) with a null operand. Added at the end for rev-compatibility.
    NullArithmetic(String),

    /// Invalid call arguments, with what's wrong (catch blocks still see 'FuncArgs'). Added at the end for rev-compatibility.
    FuncArgsInfo(String),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(error) => {
                // parse error: expected ';'
                //   --> path/to/file.stof:7:14
                //    |
                //  7 |     let x = 5
                //    |              ^
                write!(f, "parse error: {}", error.describe())?;
                match (error.file_path(), error.location()) {
                    (Some(path), Some((line, col))) => write!(f, "\n  --> {path}:{line}:{col}")?,
                    (None, Some((line, col))) => write!(f, "\n  --> line {line}, column {col}")?,
                    (Some(path), None) => write!(f, "\n  --> {path}")?,
                    (None, None) => {},
                }
                if let Some(info) = &error.info {
                    if !info.frame.is_empty() { write!(f, "\n{}", info.frame)?; }
                }
                Ok(())
            },
            Self::Located(error, stack) => {
                write!(f, "{error}")?;
                if !stack.is_empty() { write!(f, "\n{stack}")?; }
                Ok(())
            },
            _ => write!(f, "{}", self.message()),
        }
    }
}


/// Library names in error variant names (Ex. StrSplit -> Str.split).
const ERROR_LIBS: [&str; 19] = ["Prompt", "Image", "Blob", "Data", "Time", "Http", "List", "Map", "Set", "Num", "Obj", "Str", "Tup", "Ver", "Std", "Pdf", "Fs", "Fn", "Age"];

/// "the value is a number" style description of a library's value type.
fn lib_value(lib: &str) -> Option<&'static str> {
    Some(match lib {
        "Empty" => "null",
        "Str" => "a string",
        "Num" => "a number",
        "Bool" => "a boolean",
        "List" => "a list",
        "Map" => "a map",
        "Set" => "a set",
        "Tup" => "a tuple",
        "Blob" => "a blob",
        "Obj" => "an object",
        "Fn" => "a function",
        "Ver" => "a version",
        "Prompt" => "a prompt",
        "Promise" => "a promise",
        "Data" => "data",
        _ => return None,
    })
}

/// "ToRFC3339" -> "to_rfc3339", "JSONStringImport" -> "json_string_import"
fn snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (index, ch) in chars.iter().enumerate() {
        if ch.is_uppercase() && index > 0 {
            let prev = chars[index - 1];
            let next_lower = chars.get(index + 1).map(|next| next.is_lowercase()).unwrap_or(false);
            if prev.is_lowercase() || prev.is_ascii_digit() || (prev.is_uppercase() && next_lower) { out.push('_'); }
        }
        out.extend(ch.to_lowercase());
    }
    out
}

impl Error {
    /// The error without host location info (see `Error::Located`).
    pub fn inner(&self) -> &Self {
        match self {
            Self::Located(error, _) => error.inner(),
            error => error,
        }
    }

    /// Short error code (the variant name, Ex. "FuncDne").
    /// This is also what a Stof catch block receives for runtime errors (stable for code that checks it).
    pub fn code(&self) -> String {
        let debug = format!("{:?}", self.inner());
        match debug.find('(') {
            Some(index) => debug[..index].to_string(),
            None => debug,
        }
    }

    /// The value a Stof catch block receives: thrown values as is, other errors as their debug form
    /// (Ex. "FuncDne(\"Num.split\")"), which existing code compares against.
    pub fn catch_value(&self) -> Val {
        match self {
            Self::Thrown(val) => val.clone(),
            Self::ParseError(_) => Val::Str(self.to_string().into()),
            Self::Located(error, _) => error.catch_value(),
            Self::FuncArgsInfo(_) => Val::Str("FuncArgs".into()),
            _ => Val::Str(format!("{:?}", self).into()),
        }
    }

    /// Human readable message.
    pub fn message(&self) -> String {
        match self {
            Self::Thrown(val) => match val {
                Val::Str(message) => message.to_string(),
                Val::Null | Val::Void => "error thrown (null)".into(),
                other => format!("error thrown: {other:?}"),
            },
            Self::AssertFailed(msg) |
            Self::AssertNotFailed(msg) |
            Self::AssertEqFailed(msg) |
            Self::AssertNotEqFailed(msg) => format!("assertion failed: {msg}"),

            Self::FuncDne(path) => {
                if let Some((lib, func)) = path.split_once("::") {
                    return format!("the {lib} library has no function '{func}'");
                }
                if let Some(func) = path.strip_prefix("Std.") {
                    return format!("function '{func}' not found (no variable, standard library function, or function at that path)");
                }
                if let Some((lib, func)) = path.split_once('.') {
                    if !func.contains('.') {
                        if lib == "Empty" {
                            return format!("cannot call '{func}' on null (the value is null or missing - check the name or path)");
                        }
                        if let Some(value) = lib_value(lib) {
                            return format!("'{func}' is not a function for {value} ({lib} library)");
                        }
                    }
                }
                format!("function '{path}' not found")
            },
            Self::FuncArgs => "invalid arguments for this function call".into(),
            Self::FuncDefaultArg(error) => format!("default argument failed: {}", error.message()),
            Self::FuncInvalidReturn => "the returned value doesn't match the function's return type".into(),
            Self::CastVal(from, to) => format!("cannot cast {} to {}", from.type_of(), to.type_of()),
            Self::ObjectCastProtoDne => "cannot cast the object: its prototype type wasn't found".into(),

            Self::DeclareExisting => "a variable with this name already exists in this scope".into(),
            Self::DeclareInvalidName => "invalid variable name".into(),
            Self::AssignConst => "cannot assign to a const variable".into(),
            Self::VariableSet => "cannot set this variable".into(),
            Self::FieldReadOnlySet => "cannot set a read-only field".into(),
            Self::AssignSelf => "cannot assign to self".into(),
            Self::AssignSuper => "cannot assign to super".into(),
            Self::AssignRootNonObj => "only an object can be assigned to a root".into(),
            Self::AssignExistingRoot => "a root with this name already exists".into(),

            Self::ExecutionTimeout => "execution timed out (max execution time exceeded)".into(),
            Self::StackOverflow => "stack overflow (too many values on the stack)".into(),
            Self::CallStackOverflow => "call stack overflow (calls nested too deeply - infinite recursion?)".into(),
            Self::StackError |
            Self::SelfStackError |
            Self::NewStackError |
            Self::CallStackError |
            Self::CastStackError => format!("internal runtime error ({})", self.code()),

            Self::Truthy | Self::NotTruthy => "cannot test this value as true/false".into(),
            Self::IsNull => "unexpected null value".into(),
            Self::GreaterThan | Self::GreaterOrEq | Self::LessThan | Self::LessOrEq => "cannot compare these values".into(),
            Self::Eq => "cannot compare these values for equality".into(),
            Self::Add => "cannot add these values (incompatible types)".into(),
            Self::Sub => "cannot subtract these values (incompatible types)".into(),
            Self::Mul => "cannot multiply these values (incompatible types)".into(),
            Self::Div => "cannot divide these values (incompatible types)".into(),
            Self::Mod => "cannot take the remainder of these values (incompatible types)".into(),
            Self::AND | Self::OR | Self::XOR | Self::SHL | Self::SHR => format!("invalid operands for the bitwise {} operator", self.code()),

            Self::AwaitError(error) => format!("awaited process failed: {}", error.message()),
            Self::Custom(message) => message.to_string(),
            Self::NullArithmetic(message) => message.clone(),
            Self::FuncArgsInfo(message) => message.clone(),
            Self::NotImplemented => "not implemented".into(),
            Self::ParseError(_) => self.to_string(),
            Self::Located(error, _) => error.message(),
            Self::RelativeImportWithoutContext => "relative import without a file context".into(),
            Self::GraphFormatNotFound => "format not found".into(),
            Self::FormatFileImportNotAllowed => "file imports are not allowed here".into(),
            Self::FormatFileExportNotAllowed => "file exports are not allowed here".into(),
            Self::AgeNoMatchingKeys => "no matching keys to decrypt this data".into(),
            Self::MapConstructor(msg) => format!("map(): {msg}"),

            _ => {
                // Library and format errors: "StrSplit" -> "Str.split() failed", "JSONStringImport(msg)" -> "JSON string import failed: msg"
                let code = self.code();
                let detail = self.detail();
                for lib in ERROR_LIBS {
                    if let Some(func) = code.strip_prefix(lib) {
                        if func.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                            let call = format!("{lib}.{}()", snake_case(func));
                            return match detail {
                                Some(detail) => format!("{call} failed: {detail}"),
                                None => format!("{call} failed (invalid arguments or value)"),
                            };
                        }
                    }
                }
                match detail {
                    Some(detail) => format!("{} failed: {detail}", snake_case(&code).replace('_', " ")),
                    None => format!("{} failed", snake_case(&code).replace('_', " ")),
                }
            },
        }
    }

    /// Message carried by a variant, if any.
    fn detail(&self) -> Option<String> {
        match self {
            Self::FormatStringImportNotImplemented(s) | Self::FormatFileImportFsError(s) | Self::FormatFileExportFsError(s) |
            Self::FormatStringExportNotImplemented(s) | Self::JSONStringImport(s) | Self::JSONStringExport(s) |
            Self::TOMLStringImport(s) | Self::TOMLStringExport(s) | Self::YAMLStringImport(s) | Self::YAMLStringExport(s) |
            Self::BYTESExport(s) | Self::PKGImport(s) | Self::BSTFImport(s) | Self::BSTFExport(s) | Self::PDFImport(s) |
            Self::PDFExport(s) | Self::ImageImport(s) | Self::ImageExport(s) | Self::DocXImport(s) | Self::FsReadStringError(s) |
            Self::FsReadError(s) | Self::FsWriteError(s) | Self::StdParse(s) | Self::StdBlobify(s) | Self::StdStringify(s) |
            Self::StdHasFormat(s) | Self::StdHasLib(s) | Self::StdFormatContentType(s) | Self::HttpArgs(s) | Self::HttpSendError(s) => Some(s.clone()),
            _ => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::runtime::{Error, Val};

    #[test]
    /// Errors read as sentences; catch blocks keep the code form.
    fn readable_messages() {
        assert_eq!(Error::FuncDne("Num.split".into()).to_string(), "'split' is not a function for a number (Num library)");
        assert_eq!(Error::FuncDne("Empty.round".into()).to_string(), "cannot call 'round' on null (the value is null or missing - check the name or path)");
        assert_eq!(Error::FuncDne("Num::nope".into()).to_string(), "the Num library has no function 'nope'");
        assert_eq!(Error::FuncDne("Std.foo".into()).to_string(), "function 'foo' not found (no variable, standard library function, or function at that path)");
        assert_eq!(Error::StrSplit.to_string(), "Str.split() failed (invalid arguments or value)");
        assert_eq!(Error::TimeToRFC3339.to_string(), "Time.to_rfc3339() failed (invalid arguments or value)");
        assert_eq!(Error::JSONStringImport("bad".into()).to_string(), "json string import failed: bad");
        assert_eq!(Error::Thrown(Val::from("boom")).to_string(), "boom");
        assert_eq!(Error::AssignConst.to_string(), "cannot assign to a const variable");
        assert_eq!(Error::AssignConst.catch_value(), Val::from("AssignConst"));
        assert_eq!(Error::FuncDne("x".into()).code(), "FuncDne");
        let located = Error::Located(Box::new(Error::AssignConst), "  at root.main (1:1)".into());
        assert_eq!(located.to_string(), "cannot assign to a const variable\n  at root.main (1:1)");
        assert_eq!(located.code(), "AssignConst");
    }

    #[test]
    /// Argument errors say what's wrong, and catch blocks still see 'FuncArgs'.
    fn argument_errors() {
        use crate::{model::Graph, runtime::Runtime};
        let mut graph = Graph::default();
        graph.parse_stof_src(r#"
            fn total(units: int, rate: float = 2) -> float { units * rate }
            #[main]
            fn needs_arg(v: int) { }
        "#, None, Default::default()).unwrap();
        let error = Runtime::call(&mut graph, "total", vec![]).unwrap_err();
        assert_eq!(error.inner().to_string(), "missing argument 'units' for total(units: int, rate?: float)");
        assert_eq!(error.inner().catch_value(), Val::from("FuncArgs"));
        let error = Runtime::call(&mut graph, "total", vec![Val::from(1i64), Val::from(2i64), Val::from(3i64)]).unwrap_err();
        assert_eq!(error.inner().to_string(), "too many arguments (3 given, 2 expected) for total(units: int, rate?: float)");

        // a main function that fails before its call starts is still reported
        let report = Runtime::run(&mut graph, None, true).unwrap_err();
        assert!(report.contains("needs_arg"), "{report}");
        assert!(report.contains("missing argument 'v' for needs_arg(v: int)"), "{report}");
    }
}
