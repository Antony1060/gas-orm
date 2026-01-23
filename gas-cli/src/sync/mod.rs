pub mod diff;
mod helpers;
pub mod variants;

pub struct MigrationScript {
    pub forward: String,
    pub backward: String,
}
