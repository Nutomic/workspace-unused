use crate::rustdoc::{ApiItemSpan, ItemKind};

#[derive(Debug, Clone)]
pub struct ItemDocsMerged {
    pub name: String,
    pub span: ApiItemSpan,
    pub visibility: String,
    pub kind: ItemKind,
}
