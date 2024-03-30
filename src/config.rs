use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::{self, File},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub projects: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub location: PathBuf,
    pub fixmes: Vec<Fixme>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fixme {
    pub message: String,
    pub location: PathBuf,
    pub created: DateTime<Utc>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ListScope {
    Directory,
    Project,
    All,
}

impl fmt::Display for Fixme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) {}",
            self.created.naive_local(),
            self.location.to_str().unwrap(),
            self.message
        )
    }
}

impl Config {
    pub fn load() -> std::io::Result<Self> {
        let file_path = get_config_path()?;
        let contents = fs::read_to_string(file_path)?;
        match toml::from_str(&contents) {
            Ok(conf) => Ok(conf),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
        }
    }

    pub fn list_fixmes(&self, scope: ListScope) -> std::io::Result<Vec<(&PathBuf, &Fixme)>> {
        let cur_dir = std::env::current_dir()?;
        let cur_dir = std::fs::canonicalize(cur_dir)?;
        let mut fixmes: Vec<(&PathBuf, &Fixme)> = vec![];
        for project in &self.projects {
            if scope == ListScope::All {
                for fixme in &project.fixmes {
                    fixmes.push((&project.location, fixme));
                }
            } else if scope == ListScope::Project && cur_dir.starts_with(&project.location) {
                for fixme in &project.fixmes {
                    fixmes.push((&project.location, fixme));
                }
            } else if scope == ListScope::Directory && cur_dir.starts_with(&project.location) {
                for fixme in &project.fixmes {
                    let fixme_path = project.location.join(&fixme.location);
                    if cur_dir == fixme_path {
                        fixmes.push((&project.location, fixme));
                    }
                }
            }
        }
        fixmes.sort_by_key(|f| f.1.created);
        fixmes.reverse();
        Ok(fixmes)
    }
}

fn app_name() -> String {
    let app_name = std::env::current_exe().expect("application to have a name");
    let s = app_name.file_name().expect("file path to have a base name");
    String::from(
        s.to_str()
            .expect("os string to have a String representation"),
    )
}

fn get_config_path() -> std::io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(app_name())?;
    Ok(xdg_dirs.get_config_file("config.toml"))
}

fn create_config_file() -> std::io::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(app_name())?;
    let result_path = xdg_dirs.place_config_file("config.toml");
    if let Err(err) = &result_path {
        if err.kind() == std::io::ErrorKind::AlreadyExists {
            println!("Configuration file already exists.");
            return Ok(());
        }
    }
    let result_path = result_path?;
    if !result_path.exists() {
        println!("Creating config file: {:?}", result_path);
        File::create(result_path).map(|_| ())
    } else {
        println!("File already exists");
        Ok(())
    }
}

pub fn init() -> std::io::Result<()> {
    println!("Initializing configuration...");
    create_config_file()
}
