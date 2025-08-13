//! A mergepen repository, which adds extra functionality on top of an
//! Automerge/Samod document repository.

use anyhow::Result;
use samod::DocumentId;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};
use std::{
    fmt::{self, Display},
    marker::PhantomData,
    path::PathBuf,
    str::FromStr,
};

use crate::doc_repo::DocRepo;

/// Copy-paste: https://users.rust-lang.org/t/serde-fromstr-on-a-field/99457/5
///
/// DocumentId can be deserialized from a string by serde, but it requires a
/// written-out UUID, not the base58check preferred in automerge stuff.
fn deserialize_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
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

    pub fn doc_repo_root(&self) -> PathBuf {
        let mut p = self.repo_root.clone();
        p.push("docs");
        p
    }

    pub async fn load_doc_repo(&self) -> Result<DocRepo> {
        DocRepo::load(self.doc_repo_root()).await
    }
}
