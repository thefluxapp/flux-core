use anyhow::Error;

mod app;

#[tokio::main]
async fn main() -> Result<(), Error> {
    flux_lib::tracing::init()?;

    app::run().await?;

    Ok(())
}
