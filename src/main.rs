#![windows_subsystem = "windows"]

use frontend::GUI;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_unwrap::ResultExt;

static SAVEINFO_LUA: &[u8] = include_bytes!("../scripts/save_info.lua");

pub fn main() -> frontend::Result {
    setup();
    info!("Starting up yama...");

    let config_path = confy::get_configuration_file_path("yama", "config")
        .unwrap()
        .parent()
        .unwrap()
        .join("scripts");

    if config_path.join("save_info.lua").metadata().is_err() {
        std::fs::create_dir_all(&config_path).unwrap_or_log();
        std::fs::write(config_path.join("save_info.lua"), SAVEINFO_LUA).unwrap_or_log();
    }

    GUI::execute()
}

fn setup() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            "error,yama=info,backend=info,frontend=info,bridge=info",
        )
    }

    let config_path = confy::get_configuration_file_path("yama", "config")
        .unwrap()
        .parent()
        .unwrap()
        .join("logs");

    std::fs::create_dir_all(&config_path).unwrap();

    let file_appender = tracing_appender::rolling::daily(&config_path, "yama.log");
    let timer = tracing_subscriber::fmt::time::UtcTime::new(time::macros::format_description!(
        "[day]/[month]/[year] - [hour]:[minute]:[second] UTC"
    ));

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .with_timer(timer)
        .with_writer(file_appender)
        .init();
}
