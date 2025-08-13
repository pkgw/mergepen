//! A mergepen repository, which adds extra functionality on top of an
//! Automerge/Samod document repository.

use anyhow::Result;
use samod::{ConnDirection, DocumentId, PeerId, Samod, storage::TokioFilesystemStorage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::doc_repo::DocRepo;

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoConfig {
    root_folder_id: DocumentId,
}

pub struct Repository {
    work_root: PathBuf,
    repo_root: PathBuf,
}

impl Repository {
    pub fn get() -> Result<Self> {
        let work_root = PathBuf::from(".");
        let repo_root = {
            let mut p = work_root.clone();
            p.push(".mergepen");
            p
        };

        let mut docs_root = repo_root.clone();
        docs_root.push("docs");
        std::fs::create_dir_all(docs_root)?;

        Ok(Repository {
            work_root,
            repo_root,
        })
    }

    pub fn doc_repo_root(&self) -> PathBuf {
        let mut p = self.repo_root.clone();
        p.push("docs");
        p
    }

    pub async fn load_doc_repo(&self) -> Result<DocRepo> {
        DocRepo::load(self.doc_repo_root()).await
    }
}
