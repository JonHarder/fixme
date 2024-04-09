use crate::config::Config;

#[derive(Debug)]
pub struct FixId {
    pub project_id: u8,
    pub fixme_id: u8,
}

pub fn fix(_conf: &mut Config, id: FixId) {
    println!("Fixing id: {id:?}");
}
