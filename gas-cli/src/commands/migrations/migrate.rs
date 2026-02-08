use crate::commands::migrations::{MigrateCount, MigrateOptions, MigrationArgs};
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::sync::helpers::diff::ChangeDirection;
use crate::sync::MigrationScript;
use crate::util::styles::{STYLE_ERR, STYLE_OK, STYLE_WARN};
use gas::connection::{PgConnection, PgRawExecutor, PgTransaction};
use gas::internals::SqlQuery;
use gas::ModelOps;
use gas_shared::migrations::parse_migrations_from_dir;
use std::borrow::Cow;
use std::cmp::{max, min};
use tracing::instrument::WithSubscriber;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub struct MigrationMigrateCommand {
    pub(super) args: MigrationArgs,
    pub(super) migrate_options: MigrateOptions,
}

// visibility of Model struct is a little weird so we pub(super)
// NOTE: maybe add some hashing things idk
#[gas::model(table_name = "__gas_orm_migrations_meta", mod_name = "meta")]
pub(super) struct MigrationsMeta {
    pub(super) count: i64,
}

async fn connect_db() -> GasCliResult<PgConnection> {
    const VAR: &str = "PG_DATABASE_URL";

    let url = std::env::var(VAR).map_err(|_| GasCliError::MissingEnvError(Cow::from(VAR)))?;

    let connection = PgConnection::new_connection_pool(url).await?;

    Ok(connection)
}

async fn handle_one_migration(
    transaction: &mut PgTransaction,
    migration: MigrationScript,
    direction: &ChangeDirection,
) -> GasCliResult<()> {
    let gas_tracing_subscriber = {
        let filter = EnvFilter::new(format!("gas={}", Level::TRACE));

        FmtSubscriber::builder()
            .with_env_filter(filter)
            .pretty()
            .finish()
    };

    let full_sql = match direction {
        ChangeDirection::Forward => migration.forward,
        ChangeDirection::Backward => migration.backward,
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

        Ok::<(), GasCliError>(())
    }
    .with_subscriber(gas_tracing_subscriber)
    .await;

    if exec_result.is_err() {
        transaction.discard().await?;
    }

    exec_result
}

async fn migrate_database(
    connection: PgConnection,
    migrations: &[MigrationScript],
    count: i64,
    direction: ChangeDirection,
) -> GasCliResult<()> {
    assert_ne!(count, 0);
    assert!(
        (count > 0 && direction == ChangeDirection::Forward)
            || (count < 0 && direction == ChangeDirection::Backward)
    );

    let current = {
        let mut current = meta::Model::query().find_one(&connection).await?;

        if current.is_none() {
            meta::Def!().insert(&connection).await?;

            current = meta::Model::query().find_one(&connection).await?;
        }

        current.expect("migrations meta does not exist")
    };

    if current.count > migrations.len() as i64 {
        println!(
            "{} ({} > {})",
            STYLE_WARN.apply_to("Database seems to be in the future?"),
            current.count,
            migrations.len(),
        );

        return Err(GasCliError::GeneralFailure);
    }

    if current.count == migrations.len() as i64 && direction == ChangeDirection::Forward {
        println!(
            "{}",
            STYLE_OK.apply_to("Nothing to do, database already up to date"),
        );

        return Ok(());
    }

    if current.count == 0 && migrations.is_empty() {
        println!("{}", STYLE_WARN.apply_to("No migrations to run"));

        return Err(GasCliError::GeneralFailure);
    }

    if current.count == 0 && direction == ChangeDirection::Backward {
        println!("{}", STYLE_WARN.apply_to("No migrations to run"));

        return Err(GasCliError::GeneralFailure);
    }

    let mut start_idx = current.count;
    let mut end_idx = start_idx + count;
    if direction == ChangeDirection::Backward {
        start_idx -= 1;
    }

    if direction == ChangeDirection::Forward {
        end_idx -= 1;
    }

    start_idx = max(0, min(start_idx, (migrations.len() as i64) - 1));
    end_idx = max(0, min(end_idx, (migrations.len() as i64) - 1));

    let mut meta_model = current.clone();

    loop {
        println!(
            "\n{}: {}",
            STYLE_OK.apply_to(format!("Running migration ({direction})")),
            start_idx + 1
        );

        {
            let mut transaction = connection.transaction().await?;

            handle_one_migration(
                &mut transaction,
                // don't like the clone but oh well
                migrations[start_idx as usize].clone(),
                &direction,
            )
            .await?;

            meta_model.count += count.signum();
            meta_model.update(&mut transaction).await?;

            transaction.save().await?;
        }

        if start_idx == end_idx {
            break;
        }

        start_idx += count.signum();
    }

    println!("{}", STYLE_OK.apply_to("Operation completed successfully!"));

    Ok(())
}

#[async_trait::async_trait]
impl Command for MigrationMigrateCommand {
    async fn execute(&self) -> GasCliResult<()> {
        if self.migrate_options.back && self.migrate_options.count.is_none() {
            println!(
                "{}",
                STYLE_ERR.apply_to("You must specify how many migrations to run (see --count option) when going backwards"),
            );

            return Err(GasCliError::GeneralFailure);
        }

        let migrations = parse_migrations_from_dir(self.args.migrations_dir_path())?
            .into_iter()
            .map(|(forward, backward)| MigrationScript { forward, backward })
            .collect::<Vec<_>>();

        if migrations.len() > (i64::MAX as usize) {
            println!("{}", STYLE_ERR.apply_to("Too many migrations lol"),);

            return Err(GasCliError::GeneralFailure);
        }

        let migrate_count = self
            .migrate_options
            .count
            .as_ref()
            .unwrap_or(&MigrateCount::All)
            .as_signed_count(self.migrate_options.back, migrations.len());

        let connection = connect_db().await?;

        meta::Model::create_table(&connection, true).await?;

        migrate_database(
            connection,
            &migrations,
            migrate_count,
            if migrate_count > 0 {
                ChangeDirection::Forward
            } else {
                ChangeDirection::Backward
            },
        )
        .await?;

        Ok(())
    }
}
