use crate::config::Config;
use crate::config::FixId;

pub fn fix(conf: &mut Config, id: FixId) -> std::io::Result<()> {
    let fixme = conf
        .projects
        .get_mut(id.project_id)
        .and_then(|p| p.get_fixme_mut(id.fixme_id))
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid project or fixme id. See 'fixme list'",
            )
        })?;
    fixme.complete();
    conf.save()?;
    Ok(())
}
