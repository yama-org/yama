use frontend::{Frontend, Result};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use tracing::info;

static SAVEINFO_LUA: &[u8] = include_bytes!("../scripts/save_info.lua");
static DEFAULT_THEME: &[u8] = include_bytes!("../res/iced.json");

static CFG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    confy::get_configuration_file_path("yama", "config")
        .expect("No configuration path found.")
        .parent()
        .expect("No valid configuration path found.")
        .to_path_buf()
});

pub fn main() -> Result<()> {
    #[cfg(target_os = "windows")]
    windows_setup()?;

    setup_logger();
    info!("Starting up...\n{:-^1$}", " yama ", 80);

    let config = confy::load("yama", "config")?;

    if !CFG_PATH.join("scripts/save_info.lua").is_dir() {
        std::fs::create_dir_all(CFG_PATH.join("scripts"))?;
        std::fs::write(CFG_PATH.join("scripts/save_info.lua"), SAVEINFO_LUA)?;
    }

    if !CFG_PATH.join("themes").is_dir() {
        std::fs::create_dir_all(CFG_PATH.join("themes"))?;
        std::fs::write(CFG_PATH.join("themes/iced.json"), DEFAULT_THEME)?;
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

    let logs_path = CFG_PATH.join("logs");

    std::fs::create_dir_all(&logs_path).unwrap();

    let file_appender = tracing_appender::rolling::daily(&logs_path, "yama.log");
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

#[cfg(target_os = "windows")]
fn windows_setup() -> Result<()> {
    use windows::{
        core::PCWSTR,
        Win32::{
            System::Console::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS},
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE},
        },
    };

    unsafe {
        FreeConsole();
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    let _icon = unsafe {
        LoadImageW(
            GetModuleHandleW(None)?,
            PCWSTR(1 as _), // Value must match the `nameID` in the .rc script
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        )
    }?;

    Ok(())
}
