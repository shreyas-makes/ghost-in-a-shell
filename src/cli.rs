use clap::{Args, Parser, Subcommand};

use crate::Result;
use crate::ghostty_adapter::AppleScriptGhosttyAdapter;
use crate::recovery::RecoverySummary;
use crate::store::{AppPaths, SnapshotStore};
use crate::tui::run_tui;

#[derive(Debug, Parser)]
#[command(name = "ghost-in-a-shell")]
#[command(about = "Ghostty workspace continuity for macOS", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Tui,
    Save(SaveArgs),
    List,
    Restore(RestoreArgs),
    Switch(RestoreArgs),
    Rename(RenameArgs),
    Delete(DeleteArgs),
}

#[derive(Debug, Args)]
pub struct SaveArgs {
    pub name: String,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct RestoreArgs {
    pub name: String,
    #[arg(long)]
    pub run: bool,
}

#[derive(Debug, Args)]
pub struct RenameArgs {
    pub old: String,
    pub new: String,
}

#[derive(Debug, Args)]
pub struct DeleteArgs {
    pub name: String,
}

pub fn run_cli(cli: Cli, paths: AppPaths) -> Result<()> {
    let store = SnapshotStore::new(paths);
    let adapter = AppleScriptGhosttyAdapter::new();

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => run_tui(store, adapter),
        Commands::Save(args) => {
            let snapshot = adapter.capture_workspace(&args.name)?;
            store.save(&snapshot, args.force)?;
            println!(
                "saved {} ({}) with {} terminals",
                snapshot.name,
                snapshot.slug,
                snapshot.terminal_count()
            );
            Ok(())
        }
        Commands::List => {
            for snapshot in store.list()? {
                println!(
                    "{}\t{}\t{}\t{}",
                    snapshot.name,
                    snapshot.updated_at.to_rfc3339(),
                    snapshot.terminal_count(),
                    snapshot.summary_line()
                );
            }
            Ok(())
        }
        Commands::Restore(args) | Commands::Switch(args) => {
            let snapshot = store.load(&args.name)?;
            let restored = adapter.restore_workspace(&snapshot, args.run)?;
            let summary = RecoverySummary::from_snapshot(&restored);
            println!("{}", summary.render_text());
            Ok(())
        }
        Commands::Rename(args) => {
            let renamed = store.rename(&args.old, &args.new)?;
            println!("renamed snapshot to {} ({})", renamed.name, renamed.slug);
            Ok(())
        }
        Commands::Delete(args) => {
            store.delete(&args.name)?;
            println!("deleted {}", args.name);
            Ok(())
        }
    }
}
