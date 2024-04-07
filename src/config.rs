use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub projects: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub location: PathBuf,
    pub fixmes: Vec<Fixme>,
}

impl Project {
    pub fn name(&self) -> &str {
        let pname = self.location.file_name().unwrap();
        pname.to_str().unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub fn list_fixmes(&self, scope: ListScope) -> std::io::Result<Vec<(&str, &Fixme)>> {
        let cur_dir = std::env::current_dir()?;
        let cur_dir = std::fs::canonicalize(cur_dir)?;
        let mut fixmes: Vec<(&str, &Fixme)> = vec![];
        for project in &self.projects {
            if (scope == ListScope::All)
                || (scope == ListScope::Project && cur_dir.starts_with(&project.location))
            {
                for fixme in &project.fixmes {
                    fixmes.push((&project.name(), fixme));
                }
            } else if scope == ListScope::Directory && cur_dir.starts_with(&project.location) {
                for fixme in &project.fixmes {
                    let fixme_path = project.location.join(&fixme.location);
                    if cur_dir == fixme_path {
                        fixmes.push((&project.name(), fixme));
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
    s.to_str()
        .expect("os string to have a String representation")
        .to_string()
}

fn get_config_path() -> std::io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(app_name())?;
    Ok(xdg_dirs.get_config_file("config.toml"))
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

/// Creates a project in the configuration file with location set to current directory.
fn initialize_project(conf: &mut Config) -> std::io::Result<()> {
    let current_dir = std::env::current_dir()?;
    let current_dir = std::fs::canonicalize(current_dir)?;
    if conf
        .projects
        .iter()
        .filter(|p| p.location == current_dir)
        .count()
        == 0
    {
        println!("No projects found for: {:?}.", current_dir);
        conf.projects.push(Project {
            location: current_dir.clone(),
            fixmes: vec![],
        });
        conf.save()?;
        println!(
            "Updated configuration with new project for: {:?}",
            current_dir
        );
        return Ok(());
    } else {
        println!("Project for this directory already exists.");
    }
    Ok(())
}

pub fn init() -> std::io::Result<()> {
    println!("Initializing configuration...");
    create_config_file()?;
    let mut config = Config::load()?;
    initialize_project(&mut config)
}

pub fn add(conf: &mut Config, message: &str) -> std::io::Result<Fixme> {
    let dir = std::env::current_dir()?;
    let dir = std::fs::canonicalize(dir)?;
    let fixme = Fixme {
        message: message.to_string(),
        location: dir,
        created: Utc::now(),
    };
    for project in &mut conf.projects {
        for path in fixme.location.ancestors() {
            if path == project.location {
                project.fixmes.push(fixme.clone());
                conf.save()?;
                return Ok(fixme);
            }
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "No project initialized for this directory. run 'fixme init' first",
    ))
}

#[derive(Debug)]
pub struct FixId {
    pub project_id: u8,
    pub fixme_id: u8,
}

pub fn fix(_conf: &mut Config, id: FixId) {
    println!("Fixing id: {id:?}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_fixme_when_no_matching_project_is_err() {
        let mut conf = Config::new();
        assert!(add(&mut conf, "foobar").is_err());
    }

    #[test]
    fn add_fixme_when_fixme_in_project_location_is_ok() -> std::io::Result<()> {
        let mut conf = Config::new();
        let dir = std::env::current_dir()?;
        conf.projects.push(Project {
            location: dir,
            fixmes: vec![],
        });
        let result = add(&mut conf, "");
        assert!(result.is_ok());
        Ok(())
    }
}
