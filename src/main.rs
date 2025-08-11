use anyhow::{Result, anyhow};
use clap::Parser;

mod repo;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Subcommands,
}

#[derive(Parser, Debug)]
enum Subcommands {
    /// Dump a document to stdout as automerge JSON.
    Cat(CatCommand),

    /// Fetch a document from a remote repository.
    Fetch(FetchCommand),

    /// Get the heads of a document.
    Heads(HeadsCommand),
}

impl Subcommands {
    async fn exec(self) -> Result<()> {
        match self {
            Subcommands::Cat(a) => a.exec().await,
            Subcommands::Fetch(a) => a.exec().await,
            Subcommands::Heads(a) => a.exec().await,
        }
    }
}

#[derive(Parser, Debug)]
#[command()]
struct CatCommand {
    #[arg()]
    docid: String,
}

impl CatCommand {
    async fn exec(self) -> Result<()> {
        let docid = self.docid.parse()?;
        let repo = repo::Repository::load().await?;
        let maybe_doc = repo.samod().find(docid).await?;
        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;

        doc.with_document(|md| {
            let content = md.hydrate(None);
            // TODO: pretty-print!
            println!("{content:?}");
        });

        repo.stop().await;
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command()]
struct FetchCommand {
    #[arg()]
    url: String,

    #[arg()]
    peerid: String,

    #[arg()]
    docid: String,
}

impl FetchCommand {
    async fn exec(self) -> Result<()> {
        let docid = self.docid.parse()?;

        let repo = repo::Repository::load().await?;
        repo.connect_websocket(&self.url, self.peerid.as_ref())
            .await?;

        let maybe_doc = repo.samod().find(docid).await?;

        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;
        println!("{}", doc.url());

        repo.stop().await;
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command()]
struct HeadsCommand {
    #[arg()]
    docid: String,
}

impl HeadsCommand {
    async fn exec(self) -> Result<()> {
        let docid = self.docid.parse()?;
        let repo = repo::Repository::load().await?;
        let maybe_doc = repo.samod().find(docid).await?;
        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;

        doc.with_document(|md| {
            for h in md.get_heads() {
                println!("{h}");
            }
        });

        repo.stop().await;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    //tracing_subscriber::fmt::init();

    if let Err(err) = args.cmd.exec().await {
        eprintln!("fatal error: {}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("caused by: {}", cause));
        std::process::exit(1);
    }
}
