use crate::config::{Config, Fixme, Project};

#[derive(PartialEq, Eq, Debug)]
pub enum ListScope {
    Directory,
    Project,
    All,
}

pub fn list(conf: &Config, scope: ListScope) -> std::io::Result<Vec<(&Project, &Fixme)>> {
    let cur_dir = std::env::current_dir()?;
    let cur_dir = std::fs::canonicalize(cur_dir)?;
    let mut fixmes: Vec<(&Project, &Fixme)> = vec![];
    for project in &conf.projects {
        if (scope == ListScope::All)
            || (scope == ListScope::Project && project.is_path_in_project(&cur_dir))
        {
            for fixme in project.fixmes() {
                fixmes.push((&project, fixme));
            }
        } else if scope == ListScope::Directory && project.is_path_in_project(&cur_dir) {
            for fixme in project.fixmes() {
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
