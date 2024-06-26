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
    pub fixmes: Vec<Fixme>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fixme {
    pub message: String,
    pub location: PathBuf,
    pub created: DateTime<Utc>,
    pub status: FixmeStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FixmeStatus {
    Active,
    Complete,
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

    pub fn get_fixme(&self, i: usize) -> Option<&Fixme> {
        self.fixmes.get(i)
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
            status: FixmeStatus::Active,
        }
    }

    pub fn complete(&mut self) {
        self.status = FixmeStatus::Complete;
    }

    pub fn is_active(&self) -> bool {
        self.status == FixmeStatus::Active
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

#[derive(Debug, Clone, Copy)]
pub struct FixId {
    pub project_id: usize,
    pub fixme_id: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub enum IndexError {
    ProjectIdOutOfBounds,
    FixmeIdOutOfBounds,
}

impl From<IndexError> for std::io::Error {
    fn from(value: IndexError) -> Self {
        let msg = format!("{:?}", value);
        std::io::Error::new(std::io::ErrorKind::InvalidInput, msg)
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
        std::fs::write(path, contents)
    }

    pub fn get_fixme_mut(&mut self, id: FixId) -> Result<&mut Fixme, IndexError> {
        self.projects
            .get_mut(id.project_id)
            .ok_or(IndexError::ProjectIdOutOfBounds)
            .and_then(|p| {
                p.fixmes
                    .get_mut(id.fixme_id)
                    .ok_or(IndexError::FixmeIdOutOfBounds)
            })
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_fixme_returns_fixme_whith_valid_index() {
        let id = FixId {
            project_id: 0,
            fixme_id: 0,
        };
        let mut conf = Config::new();
        let mut project = Project::new(PathBuf::new());
        project.fixmes.push(Fixme::new(PathBuf::new(), ""));
        conf.projects.push(project);

        assert!(conf.get_fixme_mut(id).is_ok());
    }

    #[test]
    fn get_fixme_return_project_error_with_invalid_project_id() {
        let id = FixId {
            project_id: 0,
            fixme_id: 0,
        };
        let mut conf = Config::new();

        assert!(conf
            .get_fixme_mut(id)
            .is_err_and(|e| e == IndexError::ProjectIdOutOfBounds));
    }
}
