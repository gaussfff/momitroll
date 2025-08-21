use tracing::{Level, subscriber::set_global_default};
use tracing_subscriber::{Registry, filter::LevelFilter, fmt, layer::SubscriberExt};

pub fn init_logger() -> anyhow::Result<()> {
    set_global_default(
        Registry::default()
            .with(
                fmt::layer()
                    .with_level(true)
                    .with_target(false)
                    .with_thread_ids(false)
                    .without_time()
                    .compact(), // TODO: ??
            )
            .with(LevelFilter::from_level(if cfg!(debug_assertions) {
                Level::DEBUG
            } else {
                Level::INFO
            })),
    )?;

    Ok(())
}
