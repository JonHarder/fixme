use crate::config::FixId;
use crate::config::{Config, Fixme};

pub fn fix(conf: &mut Config, id: FixId) -> Result<&Fixme, std::io::Error> {
    let fixme: &mut Fixme = conf.get_fixme_mut(id)?;
    fixme.complete();
    conf.save()?;
    let fixme = conf
        .projects
        .get(id.project_id)
        .and_then(|p| p.get_fixme(id.fixme_id));
    // This should be fine since we've already done bounds checking above and returned
    // Err if the FixId was invalid.
    Ok(fixme.unwrap())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_out_of_bounds_fix_id_is_error() {
        let mut conf = crate::config::Config::new();
        let fix_id = FixId {
            project_id: 0,
            fixme_id: 0,
        };
        assert!(fix(&mut conf, fix_id).is_err_and(|e| e.kind() == std::io::ErrorKind::InvalidInput));
    }
}
