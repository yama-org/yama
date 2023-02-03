use frontend::GUI;
use tracing::info;
use tracing_subscriber::EnvFilter;

pub fn main() -> frontend::Result {
    setup();
    info!("Starting up yama...");
    GUI::execute()
}

fn setup() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "none,yama=info,backend=info,frontend=info")
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .init();
}
