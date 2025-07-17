use std::collections::HashMap;

use serde::Deserialize;

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
}

#[derive(Deserialize, Debug)]
pub struct ApiPath {
    pub kind: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiItemSpan {
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct ItemDocsMerged {
    pub name: String,
    pub span: ApiItemSpan,
    pub visibility: String,
    pub kind: String,
}
