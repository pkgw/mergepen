//! ```
//! use serde_automerge::de::Deserializer;
//! let fd = FolderDoc::deserialize(Deserializer::new_root(md));
//! ```
//!
//! Etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PatchworkMetadata {
    suggested_import_url: String,

    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct EssayDoc {
    tags: Vec<String>,
    change_group_summaries: HashMap<String, String>,
    version_control_metadata_url: String,

    #[serde(rename = "@patchwork")]
    patchwork: PatchworkMetadata,
    discussions: HashMap<String, String>,

    content: String,
    comment_threads: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FolderItem {
    /// The title of the contained item
    name: String,

    /// The `automerge:...` URL of the item
    url: String,

    /// The type of the item: `essay`, ...

    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FolderDoc {
    tags: Vec<String>,
    change_group_summaries: HashMap<String, String>,
    version_control_metadata_url: String,

    #[serde(rename = "@patchwork")]
    patchwork: PatchworkMetadata,
    discussions: HashMap<String, String>,

    title: String,
    docs: Vec<FolderItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionControlMetadataDoc {
    is_branch_scope: bool,
    change_group_summaries: HashMap<String, String>,
}
