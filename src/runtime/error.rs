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
    ListSortWith,
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
}
impl Display for Error { // maps ToString and print to Debug
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(error) => {
                let message;
                if let Some(path) = &error.file_path {
                    message = format!("{path}\n\t{}", &error.message);
                } else {
                    message = error.message.clone();
                }
                write!(f, "{message}")
            },
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}
