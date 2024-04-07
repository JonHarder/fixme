use clap::{Args, Parser, Subcommand};
use config::ListScope;

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

impl From<Scope> for config::ListScope {
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
            let c = config::Config::load();
            match c {
                Err(err) => {
                    println!("{}", err);
                    Err(err)
                }
                Ok(conf) => {
                    for (project, fix) in conf.list_fixmes(ListScope::from(scope))? {
                        println!(
                            "[{date}] {location}: (/{folder}) {message}",
                            date = fix.created.naive_local(),
                            location = project.name(),
                            folder = config::remove_ancestors(project.location(), &fix.location)
                                .to_str()
                                .unwrap(),
                            message = fix.message,
                        );
                    }
                    Ok(())
                }
            }
        }
        Command::Init => config::init(),
    }
}
