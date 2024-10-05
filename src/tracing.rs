use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn configure_default_tracing() {
    configure_tracing(LevelFilter::ERROR)
}

pub fn configure_tracing(default_filter: LevelFilter) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false).without_time().with_level(false))
        .with(EnvFilter::builder().with_default_directive(default_filter.into()).from_env_lossy())
        .init();
}
