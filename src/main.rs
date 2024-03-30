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
        Command::Add { message: _ } => todo!(),
        Command::List { filter: _, scope } => {
            let c = config::Config::load();
            match c {
                Err(err) => {
                    println!("{}", err);
                    Err(err)
                }
                Ok(conf) => {
                    let fixmes = conf.list_fixmes(ListScope::from(scope))?;
                    for (project_location, fix) in fixmes {
                        println!(
                            "{date}: {location} {message}",
                            date = fix.created.naive_local(),
                            location = project_location.join(&fix.location).to_str().unwrap(),
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
