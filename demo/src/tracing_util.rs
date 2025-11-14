use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn setup_tracing() {
    let filter = EnvFilter::new(format!("demo={},gas={}", Level::DEBUG, Level::TRACE));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        // .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[macro_export]
macro_rules! tracing_dbg {
    ($ex:expr) => {
        tracing::debug!(value = %format!("{:#?}", $ex), "dbg");
    };
    ($prefix:literal, $ex:expr) => {
        tracing::debug!(value = %format!("{:#?}", $ex), $prefix);
    };
}
