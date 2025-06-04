use ServerOxide::logger::*;
use ServerOxide::settings::*;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let logger = Logger::new_bootstrap();
    
    let project_settings = parse_settings(cli.settings.as_deref())?;
    info!(?project_settings);
    let logger_config = LogConfig { filter: project_settings.log.filter.clone() };
    logger.reload_from_config(&logger_config)?;
    
    Ok(())
}