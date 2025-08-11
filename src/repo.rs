//! A mergepen repository.
//!

use anyhow::Result;
use openat::Dir;
use samod::{ConnDirection, PeerId, Samod, storage::TokioFilesystemStorage};

pub struct Repository {
    #[allow(dead_code)]
    root: Dir,
    samod: Samod,
}

impl Repository {
    pub async fn load() -> Result<Self> {
        let root = Dir::open(".")?;

        let builder = Samod::build_tokio();
        let storage = TokioFilesystemStorage::new(".mergepen");
        let builder = builder.with_storage(storage);
        let samod = builder.load().await;

        Ok(Repository { root, samod })
    }

    pub async fn connect_websocket<S: AsRef<str>, P: Into<PeerId>>(
        &self,
        url: S,
        pid: P,
    ) -> Result<()> {
        let (conn, _) = tokio_tungstenite::connect_async(url.as_ref()).await?;

        let conn_driver = self
            .samod
            .connect_tungstenite(conn, ConnDirection::Outgoing);

        tokio::spawn(async {
            let finished = conn_driver.await;
            eprintln!("WS connection finished: {finished:?}");
        });

        self.samod.when_connected(pid.into()).await?;

        Ok(())
    }

    pub fn samod(&self) -> &Samod {
        &self.samod
    }

    pub async fn stop(self) {
        self.samod.stop().await
    }
}
