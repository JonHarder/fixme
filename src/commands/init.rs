use crate::config::{Config, Project};

/// Creates a project in the configuration file with location set to current directory.
pub fn initialize_project(conf: &mut Config) -> std::io::Result<()> {
    let current_dir = std::env::current_dir()?;
    let current_dir = std::fs::canonicalize(current_dir)?;
    if !conf.projects.iter().any(|p| p.location() == current_dir) {
        conf.projects.push(Project::new(current_dir.clone()));
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
