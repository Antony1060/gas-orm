use crate::connection::{PgConnection, PgRawExecutor, PgTransaction};
use crate::error::GasError;
use crate::internals::SqlQuery;
use crate::{GasResult, ModelOps};
use std::borrow::Cow;
use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
use std::num::NonZeroU64;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GasMigratorError {
    #[error("migrations seem to be in the future ({0} > {1})")]
    MigrationStateInTheFuture(i64, usize),

    #[error("no migrations can run with the selected strategy: {detail}")]
    NoMigrationsToRun { detail: Cow<'static, str> },
}

#[derive(Debug, Clone)]
pub struct MigrationScript<'a> {
    forward: &'a str,
    backward: &'a str,
}

impl<'a> MigrationScript<'a> {
    pub fn new(forward: &'a str, backward: &'a str) -> Self {
        Self { forward, backward }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum MigrationsDirection {
    Forward,
    Backward,
}

impl Display for MigrationsDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forward => write!(f, "forward"),
            Self::Backward => write!(f, "backward"),
        }
    }
}

#[derive(Debug)]
pub struct Migrator<'a> {
    scripts: Box<[MigrationScript<'a>]>,
}

// macros expect gas:: as a namespace which exists
//  this is not a problem when using `gas` as a library
//  but used internally, what macros expect to be `gas::` is `crate::`
mod gas {
    pub use crate::*;
}

// visibility of Model struct is a little weird so we pub(super)
// NOTE: maybe add some hashing things idk
#[gas_macros::model(table_name = "__gas_orm_migrations_meta", mod_name = "meta")]
pub(super) struct MigrationsMeta {
    pub(super) count: i64,
}

#[derive(Debug, Clone)]
pub enum MigrateCount {
    All,
    Specific(NonZeroU64),
}

impl FromStr for MigrateCount {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "all" => Ok(MigrateCount::All),
            val => Ok(MigrateCount::Specific(NonZeroU64::from_str(val)?)),
        }
    }
}

impl MigrateCount {
    pub fn as_signed_count(&self, is_back: bool, max: usize) -> i64 {
        match self {
            MigrateCount::All if !is_back => max as i64,
            MigrateCount::All if is_back => -(max as i64),
            MigrateCount::Specific(n) if !is_back => n.get().cast_signed(),
            MigrateCount::Specific(n) if is_back => -n.get().cast_signed(),
            _ => unreachable!(),
        }
    }
}

impl<'a> Migrator<'a> {
    pub const fn from(scripts: Box<[MigrationScript<'a>]>) -> Self {
        if scripts.len() > (i64::MAX as usize) {
            panic!("too many migrations")
        }

        Migrator { scripts }
    }

    pub async fn run_migrations(
        &self,
        connection: &PgConnection,
        direction: MigrationsDirection,
        count: MigrateCount,
    ) -> GasResult<()> {
        let steps = self
            .plan_migrations(connection, direction.clone(), count)
            .await?;

        let Some(steps) = steps else {
            tracing::info!("Nothing to do, database already up to date");

            return Ok(());
        };

        for step in steps {
            tracing::info!(
                direction = direction.to_string(),
                num = step.num,
                "Running migration"
            );

            step.run().await?;
        }

        Ok(())
    }

    pub async fn plan_migrations(
        &self,
        connection: &PgConnection,
        direction: MigrationsDirection,
        count: MigrateCount,
    ) -> GasResult<Option<Vec<MigrationStep<'a>>>> {
        migrate_database(
            connection,
            &self.scripts,
            count.as_signed_count(
                direction == MigrationsDirection::Backward,
                self.scripts.len(),
            ),
            direction,
        )
        .await
    }
}

pub struct MigrationStep<'a> {
    connection: PgConnection,
    migration: MigrationScript<'a>,
    direction: MigrationsDirection,
    pub num: i64,
    update_count: i64,
}

impl<'a> MigrationStep<'a> {
    pub async fn run(&self) -> GasResult<()> {
        let mut transaction = self.connection.transaction().await?;

        handle_one_migration(&mut transaction, self.migration.clone(), &self.direction).await?;

        meta::Model {
            count: self.update_count,
        }
        .update(&mut transaction)
        .await?;

        transaction.save().await?;

        Ok(())
    }
}

async fn handle_one_migration(
    transaction: &mut PgTransaction,
    migration: MigrationScript<'_>,
    direction: &MigrationsDirection,
) -> GasResult<()> {
    let full_sql = match direction {
        MigrationsDirection::Forward => migration.forward,
        MigrationsDirection::Backward => migration.backward,
    };

    let statements = full_sql.split(';');

    let exec_result = async {
        for statement in statements {
            let trimmed = statement.trim();
            if trimmed.is_empty() {
                continue;
            }

            transaction
                .execute_raw(SqlQuery::from(trimmed), &[])
                .await?;
        }

        Ok::<(), GasError>(())
    }
    .await;

    if exec_result.is_err() {
        transaction.discard().await?;
    }

    exec_result
}

async fn migrate_database<'a>(
    connection: &PgConnection,
    migrations: &[MigrationScript<'a>],
    count: i64,
    direction: MigrationsDirection,
) -> GasResult<Option<Vec<MigrationStep<'a>>>> {
    assert_ne!(count, 0);
    assert!(
        (count > 0 && direction == MigrationsDirection::Forward)
            || (count < 0 && direction == MigrationsDirection::Backward)
    );

    let current = {
        meta::Model::create_table(connection, true).await?;

        let mut current = meta::Model::query().find_one(connection).await?;

        if current.is_none() {
            meta::Def!().insert(connection).await?;

            current = meta::Model::query().find_one(connection).await?;
        }

        current.expect("migrations meta does not exist")
    };

    if current.count > migrations.len() as i64 {
        return Err(GasError::MigratorError(
            GasMigratorError::MigrationStateInTheFuture(current.count, migrations.len()),
        ));
    }

    if current.count == migrations.len() as i64 && direction == MigrationsDirection::Forward {
        return Ok(None);
    }

    if current.count == 0 && migrations.is_empty() {
        return Err(GasError::MigratorError(
            GasMigratorError::NoMigrationsToRun {
                detail: Cow::from("no migrations defined"),
            },
        ));
    }

    if current.count == 0 && direction == MigrationsDirection::Backward {
        return Err(GasError::MigratorError(
            GasMigratorError::NoMigrationsToRun {
                detail: Cow::from("state at 0 while requested direction is backwards"),
            },
        ));
    }

    let mut start_idx = current.count;
    let mut end_idx = start_idx + count;
    if direction == MigrationsDirection::Backward {
        start_idx -= 1;
    }

    if direction == MigrationsDirection::Forward {
        end_idx -= 1;
    }

    start_idx = max(0, min(start_idx, (migrations.len() as i64) - 1));
    end_idx = max(0, min(end_idx, (migrations.len() as i64) - 1));

    let mut steps: Vec<MigrationStep> = Vec::new();

    let mut meta_model = current.clone();

    loop {
        meta_model.count += count.signum();

        steps.push(MigrationStep {
            connection: connection.clone(),
            migration: migrations[start_idx as usize].clone(),
            direction: direction.clone(),
            num: start_idx + 1,
            update_count: meta_model.count,
        });

        if start_idx == end_idx {
            break;
        }

        start_idx += count.signum();
    }

    Ok(Some(steps))
}
