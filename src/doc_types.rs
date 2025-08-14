//! ```
//! use serde_automerge::de::Deserializer;
//! let fd = FolderDoc::deserialize(Deserializer::new_root(md));
//! ```
//!
//! Etc.

use samod_core::AutomergeUrl;
use serde::{Deserialize, Serialize};
use serde_automerge::de::Deserializer;
use std::collections::HashMap;

use crate::repo::deserialize_from_str;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchworkMetadata {
    pub suggested_import_url: String,

    #[serde(rename = "type")]
    pub type_: String,
}

// No Serialize because I need to implement serialize-with here ...
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EssayDoc {
    pub tags: Vec<String>,
    pub change_group_summaries: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub version_control_metadata_url: AutomergeUrl,

    #[serde(rename = "@patchwork")]
    pub patchwork: PatchworkMetadata,
    pub discussions: HashMap<String, String>,

    pub content: String,
    pub comment_threads: HashMap<String, String>,
}

// No Serialize because I need to implement serialize-with here ...
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderItem {
    /// The title of the contained item
    pub name: String,

    /// The `automerge:...` URL of the item
    #[serde(deserialize_with = "deserialize_from_str")]
    pub url: AutomergeUrl,

    /// The type of the item: `essay`, ...

    #[serde(rename = "type")]
    pub type_: String,
}

// No Serialize because I need to implement serialize-with here ...
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderDoc {
    pub tags: Vec<String>,
    pub change_group_summaries: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub version_control_metadata_url: AutomergeUrl,

    #[serde(rename = "@patchwork")]
    pub patchwork: PatchworkMetadata,
    pub discussions: HashMap<String, String>,

    pub title: String,
    pub docs: Vec<FolderItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionControlMetadataDoc {
    pub is_branch_scope: bool,
    pub change_group_summaries: HashMap<String, String>,
}

pub trait DocHandleExt {
    fn as_folder(&self) -> Option<FolderDoc>;
    fn as_essay(&self) -> Option<EssayDoc>;
}

impl DocHandleExt for samod::DocHandle {
    fn as_folder(&self) -> Option<FolderDoc> {
        self.with_document(|md| FolderDoc::deserialize(Deserializer::new_root(md)))
            .ok()
    }

    fn as_essay(&self) -> Option<EssayDoc> {
        self.with_document(|md| EssayDoc::deserialize(Deserializer::new_root(md)))
            .ok()
    }
}
