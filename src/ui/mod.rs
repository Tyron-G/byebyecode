#[cfg(feature = "tui")]
pub mod app;
#[cfg(feature = "tui")]
pub mod components;
#[cfg(feature = "tui")]
pub mod events;
#[cfg(feature = "tui")]
pub mod layout;
#[cfg(feature = "tui")]
pub mod main_menu;
#[cfg(feature = "tui")]
pub mod themes;

#[cfg(feature = "tui")]
pub use app::App;
#[cfg(feature = "tui")]
pub use main_menu::{MainMenu, MenuResult};

use crate::target::Target;

#[cfg(feature = "tui")]
pub fn run_configurator(target: Target) -> Result<(), Box<dyn std::error::Error>> {
    App::run(target)
}

#[cfg(not(feature = "tui"))]
pub fn run_configurator(_target: Target) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("TUI feature is not enabled. Please install with --features tui");
    std::process::exit(1);
}
