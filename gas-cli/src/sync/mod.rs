pub mod diff;

#[allow(dead_code)]
pub struct MigrationScript {
    pub forward: String,
    pub backward: String,
}
