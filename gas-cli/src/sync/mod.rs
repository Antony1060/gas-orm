pub mod diff;
mod helpers;
pub mod variants;

#[allow(dead_code)]
pub struct MigrationScript {
    pub forward: String,
    pub backward: String,
}
