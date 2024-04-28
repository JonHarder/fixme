use crate::commands::list::ListScope;
use clap::{Args, Parser, Subcommand};

use crate::config::Fixme;

mod commands;
mod config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Add a new fixme in the current working directory.
    Add {
        /// The message associated with this fixme
        message: String,
    },
    /// Fix (complete) a fixme.
    Fix { project_id: u8, fixme_id: u8 },
    /// Show the fixmes local to your directory, project or all recorded.
    List {
        /// Filter fixmes
        #[arg(short, long)]
        filter: Vec<String>,

        #[command(flatten)]
        scope: Scope,
    },
    /// Initialize a fixme configuration file.
    ///
    /// This must be done to register a project before creating a fixme.
    /// If you try to create a fixme with ~fixme add~ outside of any known
    /// project, the command will fail and you will be prompted to run ~init~
    /// first.
    ///
    /// Run this command at the root of your project.
    Init,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
struct Scope {
    /// Show fixmes from whole project
    #[arg(short, long)]
    project: bool,

    /// Show all fixmes from all projects
    #[arg(short, long)]
    all: bool,
}

impl From<Scope> for commands::list::ListScope {
    fn from(value: Scope) -> Self {
        let Scope { project, all } = value;
        match (project, all) {
            (true, false) => ListScope::Project,
            (false, true) => ListScope::All,
            (false, false) => ListScope::Directory,
            _ => ListScope::Directory,
        }
    }
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Fix {
            project_id,
            fixme_id,
        } => {
            let fix_id = commands::fix::FixId {
                project_id,
                fixme_id,
            };
            let mut c = config::Config::load()?;
            commands::fix::fix(&mut c, fix_id);
            Ok(())
        }
        Command::Add { message } => {
            let mut conf = config::Config::load()?;
            let fixme = Fixme::new_in_current_dir(&message)?;
            let fixme = commands::add::add(&mut conf, fixme)?;
            println!("{}", fixme);
            conf.save()?;
            Ok(())
        }
        Command::List { filter: _, scope } => {
            let conf = config::Config::load()?;
            for indexed_fixme in commands::list::list(&conf, ListScope::from(scope))? {
                println!("{}", indexed_fixme);
            }
            Ok(())
        }
        Command::Init => config::init(),
    }
}
