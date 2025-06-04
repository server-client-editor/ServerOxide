use server_oxide::api;
use server_oxide::logger::*;
use server_oxide::settings::*;
use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let logger = Logger::new_bootstrap();
    
    let project_settings = parse_settings(cli.settings.as_deref())?;
    info!(?project_settings);
    let logger_config = LogConfig { filter: project_settings.log.filter.clone() };
    logger.reload_from_config(&logger_config)?;

    let api_v1 = warp::path("api")
        .and(warp::path("v1"))
        .and(api::v1::routes());
    
    warp::serve(api_v1)
        .run(([127, 0, 0, 1], 8443))
        .await;

    Ok(())
}