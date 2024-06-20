use crate::config::{self, Config, Fixme, Project};
use std::fmt;

#[derive(PartialEq, Eq, Debug)]
pub enum ListScope {
    Directory,
    Project,
    All,
}

pub struct IndexedFixme<'a> {
    project_id: usize,
    project: &'a Project,
    fixme_id: usize,
    fixme: &'a Fixme,
}

impl fmt::Display for IndexedFixme<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{date}] id: {project_id}-{fixme_id}, name: {location}, dir: /{folder}, {message}",
            date = self.fixme.created.naive_local(),
            location = self.project.name(),
            project_id = self.project_id,
            fixme_id = self.fixme_id,
            folder = config::remove_ancestors(self.project.location(), &self.fixme.location)
                .to_str()
                .unwrap(),
            message = self.fixme.message,
        )
    }
}

pub fn list(conf: &Config, scope: ListScope) -> std::io::Result<Vec<IndexedFixme>> {
    let cur_dir = std::env::current_dir()?;
    let cur_dir = std::fs::canonicalize(cur_dir)?;
    let mut fixmes: Vec<IndexedFixme> = vec![];
    if !conf
        .projects
        .iter()
        .any(|project| project.is_path_in_project(&cur_dir))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Directory not in any project. Run init first",
        ));
    }
    for (project_id, project) in conf.projects.iter().enumerate() {
        // TODO: refactor these to conditional branches.
        if (scope == ListScope::All)
            || (scope == ListScope::Project && project.is_path_in_project(&cur_dir))
        {
            for (fixme_id, fixme) in project.fixmes.iter().enumerate() {
                if fixme.is_active() {
                    fixmes.push(IndexedFixme {
                        project_id,
                        project,
                        fixme_id,
                        fixme,
                    })
                }
            }
        } else if scope == ListScope::Directory && project.is_path_in_project(&cur_dir) {
            for (fixme_id, fixme) in project.fixmes.iter().enumerate() {
                if fixme.location == cur_dir && fixme.is_active() {
                    fixmes.push(IndexedFixme {
                        project_id,
                        project,
                        fixme_id,
                        fixme,
                    })
                }
            }
        }
    }
    fixmes.sort_by_key(|f| f.fixme.created);
    fixmes.reverse();
    Ok(fixmes)
}
