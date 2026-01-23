use crate::error::GasError;
use crate::GasResult;

const SCRIPT_SEPARATOR: &str = "-- GAS_ORM(forward_backward_separator)";

#[allow(dead_code)]
#[derive(Debug)]
pub struct MigrationScript {
    forwards: &'static str,
    backwards: &'static str,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Migrator {
    scripts: Box<[MigrationScript]>,
}

impl Migrator {
    pub fn from_raw(scripts: &[&'static str]) -> GasResult<Self> {
        let mut parsed_scripts: Vec<MigrationScript> = Vec::with_capacity(scripts.len());
        for script in scripts {
            let (forward, backward) =
                script
                    .split_once(SCRIPT_SEPARATOR)
                    .ok_or(GasError::GeneralError(
                        "failed to parse migration script".into(),
                    ))?;

            parsed_scripts.push(MigrationScript {
                forwards: forward,
                backwards: backward,
            })
        }

        Ok(Migrator {
            scripts: parsed_scripts.into_boxed_slice(),
        })
    }
}
