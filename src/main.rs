use anyhow::{Result, anyhow};
use automerge::patches::TextRepresentation;
use clap::Parser;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

mod doc_repo;
mod doc_types;
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

    /// Dump a document using Automerge's internal format.
    Dump(DumpCommand),

    /// Export a document in the automerge binary format
    Export(ExportCommand),

    /// Fetch a document from a remote repository.
    Fetch(FetchCommand),

    /// Get the heads of a document.
    Heads(HeadsCommand),

    /// Dump patches bringing a document to its current state.
    History(HistoryCommand),
}

impl Subcommands {
    async fn exec(self) -> Result<()> {
        match self {
            Subcommands::Cat(a) => a.exec().await,
            Subcommands::Dump(a) => a.exec().await,
            Subcommands::Export(a) => a.exec().await,
            Subcommands::Fetch(a) => a.exec().await,
            Subcommands::Heads(a) => a.exec().await,
            Subcommands::History(a) => a.exec().await,
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
        let repo = repo::Repository::get()?;
        let docs = repo.load_doc_repo().await?;
        let maybe_doc = docs.samod().find(docid).await?;
        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;

        doc.with_document(|md| {
            let content = md.hydrate(None);
            // TODO: pretty-print!
            println!("{content:?}");
        });

        docs.stop().await;
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command()]
struct DumpCommand {
    #[arg()]
    docid: String,
}

impl DumpCommand {
    async fn exec(self) -> Result<()> {
        let docid = self.docid.parse()?;
        let repo = repo::Repository::get()?;
        let docs = repo.load_doc_repo().await?;
        let maybe_doc = docs.samod().find(docid).await?;
        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;

        doc.with_document(|md| {
            md.dump();
        });

        docs.stop().await;
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command()]
struct ExportCommand {
    #[arg()]
    docid: String,

    #[arg()]
    dest_path: PathBuf,
}

impl ExportCommand {
    async fn exec(self) -> Result<()> {
        let docid = self.docid.parse()?;
        let repo = repo::Repository::get()?;
        let docs = repo.load_doc_repo().await?;
        let maybe_doc = docs.samod().find(docid).await?;
        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;

        let mut dest: Box<dyn Write> = if self.dest_path == Path::new("-") {
            Box::new(std::io::stdout().lock())
        } else {
            let stream = std::fs::File::create(self.dest_path)?;
            Box::new(stream)
        };

        let serialized = doc.with_document(|md| md.save());

        dest.write_all(&serialized)?;

        docs.stop().await;
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

        let repo = repo::Repository::get()?;
        let docs = repo.load_doc_repo().await?;

        docs.connect_websocket(&self.url, self.peerid.as_ref())
            .await?;

        let maybe_doc = docs.samod().find(docid).await?;

        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;
        println!("{}", doc.url());

        docs.stop().await;
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
        let repo = repo::Repository::get()?;
        let docs = repo.load_doc_repo().await?;
        let maybe_doc = docs.samod().find(docid).await?;
        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;

        doc.with_document(|md| {
            for h in md.get_heads() {
                println!("{h}");
            }
        });

        docs.stop().await;
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command()]
struct HistoryCommand {
    #[arg()]
    docid: String,
}

impl HistoryCommand {
    async fn exec(self) -> Result<()> {
        let docid = self.docid.parse()?;
        let repo = repo::Repository::get()?;
        let docs = repo.load_doc_repo().await?;
        let maybe_doc = docs.samod().find(docid).await?;
        let doc = maybe_doc.ok_or_else(|| anyhow!("doc not found"))?;

        doc.with_document(|md| {
            let enc = md.text_encoding();

            for patch in md.current_state(TextRepresentation::String(enc)) {
                println!("{patch:?}");
            }
        });

        docs.stop().await;
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
