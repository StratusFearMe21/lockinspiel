use color_eyre::eyre;
use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub mod client;
pub mod db;
pub mod timer;

pub fn install_init_boilerplate(level_filter: Option<LevelFilter>) -> eyre::Result<()> {
    color_eyre::install()?;

    let registry = tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(tracing_subscriber::fmt::layer());

    if let Some(filter) = level_filter {
        registry.with(filter).init()
    } else {
        registry
            .with(
                EnvFilter::try_from_default_env()
                    .or_else(|_| EnvFilter::try_new("info"))
                    .unwrap(),
            )
            .init()
    }

    Ok(())
}
