use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct ApiDocs {
    pub index: HashMap<i32, ApiItem>,
    pub paths: HashMap<i32, ApiPath>,
}

#[derive(Deserialize, Debug)]
pub struct ApiItem {
    pub name: Option<String>,
    pub span: Option<ApiItemSpan>,
    pub visibility: String,
    pub inner: ApiItemInner,
}

#[derive(Deserialize, Debug)]
pub struct ApiPath {
    pub kind: String,
}

impl Default for ApiPath {
    fn default() -> Self {
        Self {
            kind: "none".to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiItemSpan {
    pub filename: String,
}

#[derive(Deserialize, Debug)]
pub enum ApiItemInner {
    #[serde(rename = "function")]
    Function(Function),
    #[serde(rename = "struct")]
    Struct(Struct),
    // TODO: all these are currently unsupported
    #[serde(
        alias = "module",
        alias = "impl",
        alias = "assoc_type",
        alias = "variant",
        alias = "struct_field",
        alias = "trait",
        alias = "type_alias",
        alias = "enum",
        alias = "constant",
        alias = "macro",
        alias = "assoc_const",
        alias = "static",
        alias = "use"
    )]
    Other(Value),
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Function {
    pub sig: FunctionSig,
    pub generics: Value,
    pub  header: Value,
    pub  has_body: bool,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Struct {
    generics: Value,
    kind: Value,
    impls: Value,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct FunctionSig {
    pub inputs: Vec<Value>,
    pub output: Value,
}
