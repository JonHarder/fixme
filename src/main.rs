use clap::{Args, Parser, Subcommand};
use config::ListScope;

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
            let fix_id = config::FixId {
                project_id,
                fixme_id,
            };
            let mut c = config::Config::load()?;
            config::fix(&mut c, fix_id);
            Ok(())
        }
        Command::Add { message } => {
            let mut conf = config::Config::load()?;
            let fixme = config::add(&mut conf, &message)?;
            println!("{}", fixme);
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
                    for (name, fix) in conf.list_fixmes(ListScope::from(scope))? {
                        println!(
                            "[{date}] {location}: /{folder:<20} {message}",
                            date = fix.created.naive_local(),
                            location = name,
                            folder = fix.location.to_str().unwrap(),
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
