//! A mergepen repository, which adds extra functionality on top of an
//! Automerge/Samod document repository.

use anyhow::{Result, anyhow};
use samod::{DocHandle, DocumentId};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};
use std::{
    collections::HashSet,
    fmt::{self, Display},
    marker::PhantomData,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{doc_repo::DocRepo, doc_types::DocHandleExt};

/// Copy-paste: https://users.rust-lang.org/t/serde-fromstr-on-a-field/99457/5
///
/// DocumentId can be deserialized from a string by serde, but it requires a
/// written-out UUID, not the base58check preferred in automerge stuff.
pub(crate) fn deserialize_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    struct Helper<S>(PhantomData<S>);
    impl<'de, S> Visitor<'de> for Helper<S>
    where
        S: FromStr,
        <S as FromStr>::Err: Display,
    {
        type Value = S;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            value.parse::<Self::Value>().map_err(DeError::custom)
        }
    }

    deserializer.deserialize_str(Helper(PhantomData))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoConfig {
    #[serde(deserialize_with = "deserialize_from_str")]
    root_folder_id: DocumentId,
}

pub struct Repository {
    work_root: PathBuf,
    repo_root: PathBuf,
    config: RepoConfig,
}

pub struct VisitNode<'a> {
    pub name: String,
    pub type_: String,
    pub doc: DocHandle,
    pub parents: Vec<String>,
    pub repo: &'a Repository,
    pub doc_repo: &'a DocRepo,
}

impl Repository {
    pub fn get() -> Result<Self> {
        let work_root = PathBuf::from(".");
        let repo_root = {
            let mut p = work_root.clone();
            p.push(".mergepen");
            p
        };

        let config_path = {
            let mut p = repo_root.clone();
            p.push("config.toml");
            p
        };
        let config_data = std::fs::read(config_path)?;
        let config = toml::from_slice(&config_data)?;

        let repo = Repository {
            work_root,
            repo_root,
            config,
        };

        std::fs::create_dir_all(repo.doc_repo_root())?;

        Ok(repo)
    }

    pub fn work_root(&self) -> &Path {
        &self.work_root
    }

    pub fn doc_repo_root(&self) -> PathBuf {
        let mut p = self.repo_root.clone();
        p.push("docs");
        p
    }

    pub async fn load_doc_repo(&self) -> Result<DocRepo> {
        DocRepo::load(self.doc_repo_root()).await
    }

    pub async fn visit<'a, F>(&'a self, doc_repo: &'a DocRepo, mut visitor: F) -> Result<()>
    where
        F: FnMut(&VisitNode<'a>),
    {
        let mut seen_ids = HashSet::new();

        let maybe_root = doc_repo
            .samod()
            .find(self.config.root_folder_id.clone())
            .await?;
        let root_doc = maybe_root.ok_or_else(|| anyhow!("root folder not found"))?;
        let root_folder = root_doc
            .as_folder()
            .ok_or_else(|| anyhow!("root doc is not a folder"))?;

        struct VisitRec {
            pub name: String,
            pub type_: String,
            pub doc_id: DocumentId,
            pub parents: Vec<String>,
        }

        let mut queue = vec![VisitRec {
            name: root_folder.title.clone(),
            type_: "folder".to_owned(),
            doc_id: self.config.root_folder_id.clone(),
            parents: vec![],
        }];

        while let Some(rec) = queue.pop() {
            seen_ids.insert(rec.doc_id.clone());

            let this_doc = doc_repo
                .samod()
                .find(rec.doc_id.clone())
                .await?
                .ok_or_else(|| anyhow!("doc not found"))?;

            if let Some(this_folder) = this_doc.as_folder() {
                let mut parents = rec.parents.clone();
                parents.push(rec.name.clone());

                for item in &this_folder.docs {
                    let doc_id = item.url.document_id().clone();

                    if seen_ids.contains(&doc_id) {
                        return Err(anyhow!("circular folder structure"));
                    }

                    queue.push(VisitRec {
                        name: item.name.clone(),
                        type_: item.type_.clone(),
                        doc_id,
                        parents: parents.clone(),
                    });
                }
            }

            visitor(&VisitNode {
                name: rec.name,
                type_: rec.type_,
                doc: this_doc,
                parents: rec.parents,
                repo: self,
                doc_repo,
            });
        }

        Ok(())
    }
}
