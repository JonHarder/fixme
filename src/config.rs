use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub projects: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    location: PathBuf,
    fixmes: Vec<Fixme>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fixme {
    pub message: String,
    pub location: PathBuf,
    pub created: DateTime<Utc>,
}

/// Given a parent path and some path, return the fragment of the path after it.
pub fn remove_ancestors(parent: &Path, path: &Path) -> PathBuf {
    let mut p = PathBuf::new();
    let mut parts = vec![];
    for ancestor in path.ancestors() {
        // println!("ancestor: {:?}, parent: {:?}", &ancestor, &parent);
        if ancestor == parent {
            break;
        } else {
            // println!("{:?}, {:?}", &ancestor, &ancestor.file_name());
            if let Some(name) = ancestor.file_name() {
                parts.push(name);
            }
        }
    }
    parts.reverse();
    for part in &parts {
        p.push(part);
    }
    p
}

impl Project {
    pub fn new(location: PathBuf) -> Self {
        Project {
            location,
            fixmes: vec![],
        }
    }

    pub fn is_path_in_project(&self, path: &Path) -> bool {
        for ancestor in path.ancestors() {
            if ancestor == self.location() {
                return true;
            }
        }
        false
    }

    pub fn location(&self) -> &Path {
        &self.location
    }

    pub fn name(&self) -> &str {
        let pname = self.location.file_name().unwrap();
        pname.to_str().unwrap()
    }

    pub fn add_fixme(&mut self, fixme: Fixme) -> &Fixme {
        self.fixmes.push(fixme);
        self.fixmes.last().unwrap()
    }

    pub fn fixmes(&self) -> &Vec<Fixme> {
        &self.fixmes
    }
}

impl Fixme {
    pub fn new_in_current_dir(message: &str) -> std::io::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let current_dir = std::fs::canonicalize(current_dir)?;
        Ok(Fixme::new(current_dir, message))
    }

    pub fn new(location: PathBuf, message: &str) -> Self {
        Fixme {
            message: message.to_string(),
            location,
            created: Utc::now(),
        }
    }
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
    pub fn new() -> Self {
        Config { projects: vec![] }
    }

    pub fn load() -> std::io::Result<Self> {
        let file_path = get_config_path()?;
        let contents = fs::read_to_string(file_path)?;
        match toml::from_str(&contents) {
            Ok(conf) => Ok(conf),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = get_config_path()?;
        let contents = toml::to_string(&self).expect("Config object to serialize to toml");
        println!("Saving config...");
        std::fs::write(path, contents)
    }
}

fn get_config_path() -> std::io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(app_name())?;
    Ok(xdg_dirs.get_config_file("config.toml"))
}

fn app_name() -> String {
    let app_name = std::env::current_exe().expect("application to have a name");
    let s = app_name.file_name().expect("file path to have a base name");
    s.to_str()
        .expect("os string to have a String representation")
        .to_string()
}

pub fn init() -> std::io::Result<()> {
    println!("Initializing configuration...");
    create_config_file()?;
    let mut config = Config::load()?;
    crate::commands::init::initialize_project(&mut config)
}

fn create_config_file() -> std::io::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(app_name())?;
    // This places the necessary parent directories to the config file itself
    let result_path = xdg_dirs.place_config_file("config.toml");
    // handle the error this could return in order to catch the AlreadyExists Error.
    // bubble up any other error
    let path = match result_path {
        Ok(p) => Ok(p),
        Err(err) => {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(get_config_path()?)
            } else {
                Err(err)
            }
        }
    }?;
    if path.exists() {
        println!("File already exists");
        Ok(())
    } else {
        Config::new().save()?;
        println!("Created empty configuration");
        Ok(())
    }
}
