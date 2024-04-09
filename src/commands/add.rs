use crate::config::{Config, Fixme};

pub fn add(conf: &mut Config, fixme: Fixme) -> std::io::Result<&Fixme> {
    for project in &mut conf.projects {
        for path in fixme.location.ancestors() {
            if path == project.location() {
                return Ok(project.add_fixme(fixme));
            }
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "No project initialized for this directory. run 'fixme init' first",
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::{Config, Project};

    #[test]
    fn add_fixme_when_no_matching_project_is_err() -> std::io::Result<()> {
        let mut conf = Config::new();
        let fixme = Fixme::new_in_current_dir("hello")?;
        assert!(add(&mut conf, fixme).is_err());
        Ok(())
    }

    #[test]
    fn add_fixme_when_fixme_in_project_location_is_ok() -> std::io::Result<()> {
        let mut conf = Config::new();
        let current_dir = std::env::current_dir()?;
        let current_dir = std::fs::canonicalize(current_dir)?;
        let project = Project::new(current_dir.clone());
        conf.projects.push(project);
        let fixme = Fixme::new_in_current_dir("")?;
        add(&mut conf, fixme).map(|_| ())
    }

    #[test]
    fn add_fixme_when_fixme_is_child_dir_of_project() -> std::io::Result<()> {
        let mut conf = Config::new();
        let current_dir = std::env::current_dir()?;
        let current_dir = std::fs::canonicalize(current_dir)?;
        let project = Project::new(current_dir.clone());
        conf.projects.push(project);
        let fixme_loc = current_dir.join("bar");
        let fixme = Fixme::new(fixme_loc, "");
        add(&mut conf, fixme).map(|_| ())
    }
}
