//! Interactive terminal UI for configuring and exploring Archipelago charts.

use std::io;

use contractors::conformance::chart::{Chart, RefChart};

pub mod app;
pub mod config;
pub mod splash;
pub mod title_art;

pub use app::{App, render};
pub use config::{GameConfig, PARAM_KINDS, ParamKind, TurnMode};
pub use splash::{SplashOutcome, SplashScreen, run_splash};

/// Runs the splash screen, setup flow, and interactive chart TUI.
///
/// Presents the main menu (`NEW GAME`, `FROM SEED`, `QUIT`), followed by the
/// mode and parameter selection screen. Once confirmed, generates a [`RefChart`]
/// using the chosen seed and parameters and opens the interactive map viewer.
///
/// # Errors
///
/// Returns an [`io::Error`] if terminal initialization, rendering, or event
/// handling fails.
pub fn tui() -> io::Result<()> {
    ratatui::run(|terminal| {
        let SplashOutcome::Start(config) = run_splash(terminal)? else {
            return Ok(());
        };

        if let Ok(chart) = RefChart::generate(config.seed, &config.params) {
            App::with_config(chart.map(), config).run(terminal)?;
        }

        Ok(())
    })
}

#[cfg(test)]
#[path = "tests/splash.rs"]
mod splash_tests;
