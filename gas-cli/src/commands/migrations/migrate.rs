use crate::commands::migrations::{MigrateOptions, MigrationArgs};
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::sync::MigrationScript;
use crate::util::styles::{STYLE_ERR, STYLE_OK, STYLE_WARN};
use gas::connection::PgConnection;
use gas::error::GasError;
use gas::migrations::{GasMigratorError, MigrateCount, MigrateDirection, Migrator};
use gas_shared::migrations::parse_migrations_from_dir;
use std::borrow::Cow;
use tracing::instrument::WithSubscriber;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub struct MigrationMigrateCommand {
    pub(super) args: MigrationArgs,
    pub(super) migrate_options: MigrateOptions,
}

async fn connect_db() -> GasCliResult<PgConnection> {
    const VAR: &str = "PG_DATABASE_URL";

    let url = std::env::var(VAR).map_err(|_| GasCliError::MissingEnvError(Cow::from(VAR)))?;

    let connection = PgConnection::new_connection_pool(url).await?;

    Ok(connection)
}

async fn do_migrate<'a>(
    connection: PgConnection,
    migrator: Migrator<'a>,
    options: &MigrateOptions,
) -> GasCliResult<()> {
    let direction = if options.back {
        MigrateDirection::Backward
    } else {
        MigrateDirection::Forward
    };

    let migrate_count = options.count.clone().unwrap_or(MigrateCount::All);

    let steps_result = migrator
        .plan_migrations(&connection, direction.clone(), migrate_count)
        .await;

    let steps = match steps_result {
        Err(GasError::MigratorError(GasMigratorError::MigrationStateInTheFuture(
            state_count,
            migrations_len,
        ))) => {
            println!(
                "{} ({} > {})",
                STYLE_WARN.apply_to("Database seems to be in the future?"),
                state_count,
                migrations_len,
            );

            return Err(GasCliError::GeneralFailure);
        }
        Err(GasError::MigratorError(GasMigratorError::NoMigrationsToRun { .. })) => {
            println!("{}", STYLE_WARN.apply_to("No migrations to run"));

            return Err(GasCliError::GeneralFailure);
        }
        Err(err) => return Err(GasCliError::from(err)),
        Ok(None) => {
            println!(
                "{}",
                STYLE_OK.apply_to("Nothing to do, database already up to date"),
            );

            return Ok(());
        }
        Ok(Some(steps)) => steps,
    };

    for step in steps {
        println!(
            "\n{}: {}",
            STYLE_OK.apply_to(format!("Running migration ({direction})")),
            step.num
        );

        let gas_tracing_subscriber = {
            let filter = EnvFilter::new(format!("gas={}", Level::TRACE));

            FmtSubscriber::builder()
                .with_env_filter(filter)
                .pretty()
                .finish()
        };

        step.run().with_subscriber(gas_tracing_subscriber).await?;
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

        let connection = connect_db().await?;

        let migrator = Migrator::from(
            migrations
                .iter()
                .map(|script| {
                    gas::migrations::MigrationScript::new(&script.forward, &script.backward)
                })
                .collect(),
        );

        do_migrate(connection, migrator, &self.migrate_options).await?;

        Ok(())
    }
}
