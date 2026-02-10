use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn setup_tracing(log_gas: bool) {
    let base_log = format!(
        "{}={},tower_http={}",
        env!("CARGO_CRATE_NAME"),
        Level::DEBUG,
        Level::DEBUG,
    );

    let filter = if log_gas {
        EnvFilter::new(format!("{base_log},gas={}", Level::TRACE))
    } else {
        EnvFilter::new(format!("{base_log},gas={}", Level::INFO))
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
