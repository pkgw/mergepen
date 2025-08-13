//! The Automerge document repository that forms the basis of most
//! mergepen processing.

use anyhow::Result;
use samod::{ConnDirection, PeerId, Samod, storage::TokioFilesystemStorage};
use std::path::Path;

pub struct DocRepo {
    samod: Samod,
}

impl DocRepo {
    pub async fn load<P: AsRef<Path>>(root: P) -> Result<Self> {
        let builder = Samod::build_tokio();
        let storage = TokioFilesystemStorage::new(root);
        let builder = builder.with_storage(storage);
        let samod = builder.load().await;

        Ok(DocRepo { samod })
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
