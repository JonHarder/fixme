use crate::config::Config;

#[derive(Debug)]
pub struct FixId {
    pub project_id: usize,
    pub fixme_id: usize,
}

pub fn fix(_conf: &mut Config, id: FixId) {
    println!("Fixing id: {id:?}");
}
