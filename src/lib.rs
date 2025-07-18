pub mod rustdoc;
pub mod search;

use crate::rustdoc::{ApiItemSpan, ItemKind};

/// Output data from `rustdoc-json` converted to a simpler format
#[derive(Debug, Clone)]
pub struct ItemDocsMerged {
    pub name: String,
    pub span: ApiItemSpan,
    pub visibility: String,
    pub kind: ItemKind,
}
