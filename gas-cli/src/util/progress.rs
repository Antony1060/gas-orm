use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;

pub fn default_spinner(message: impl Into<Cow<'static, str>>) -> ProgressBar {
    ProgressBar::new_spinner()
        .with_style(
            ProgressStyle::with_template(" {spinner} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        )
        .with_message(message)
}
