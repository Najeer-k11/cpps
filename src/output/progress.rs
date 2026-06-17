use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use super::OutputConfig;

pub fn create_spinner(config: &OutputConfig, msg: &str) -> Option<ProgressBar> {
    if !config.is_tty {
        return None;
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    Some(spinner)
}
