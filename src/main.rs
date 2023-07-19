#![windows_subsystem = "windows"]

use frontend::{Frontend, Result};
use tracing::info;

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE},
    },
};

static SAVEINFO_LUA: &[u8] = include_bytes!("../scripts/save_info.lua");

pub fn main() -> Result<()> {
    #[cfg(target_os = "windows")]
    let _icon = unsafe {
        LoadImageW(
            GetModuleHandleW(None).expect("[ERROR] - Windows icon"),
            PCWSTR(1 as _), // Value must match the `nameID` in the .rc script
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        )
    }
    .expect("[ERROR] - Windows icon");

    setup_logger();

    info!("\n{:-^1$}", " yama ", 80);
    info!("Starting up yama...");

    let config = confy::load("yama", "config")?;
    let config_path = confy::get_configuration_file_path("yama", "config")
        .unwrap()
        .parent()
        .unwrap()
        .join("scripts");

    if config_path.join("save_info.lua").metadata().is_err() {
        std::fs::create_dir_all(&config_path)?;
        std::fs::write(config_path.join("save_info.lua"), SAVEINFO_LUA)?;
    }

    Frontend::execute(config)
}

fn setup_logger() {
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
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(false)
        .with_timer(timer)
        .with_writer(file_appender)
        .init();
}
