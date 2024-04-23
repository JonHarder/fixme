use std::fmt;

use crate::config::{self, Config, Fixme, Project};

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

pub fn list(conf: &Config, scope: ListScope) -> std::io::Result<Vec<(&Project, &Fixme)>> {
    let cur_dir = std::env::current_dir()?;
    let cur_dir = std::fs::canonicalize(cur_dir)?;
    let mut fixmes: Vec<(&Project, &Fixme)> = vec![];
    for project in &conf.projects {
        if (scope == ListScope::All)
            || (scope == ListScope::Project && project.is_path_in_project(&cur_dir))
        {
            for fixme in project.active_fixmes() {
                fixmes.push((&project, fixme));
            }
        } else if scope == ListScope::Directory && project.is_path_in_project(&cur_dir) {
            for fixme in project.active_fixmes() {
                if fixme.location == cur_dir {
                    fixmes.push((&project, fixme));
                }
            }
        }
    }
    fixmes.sort_by_key(|f| f.1.created);
    fixmes.reverse();
    Ok(fixmes)
}
