use std::time::Duration;

use colored::Colorize;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use crate::emoji;

const TICK_CHARS: &str = "⠁⠂⠄⡀⢀⠠⠐⠈ ";
const TICK_DURATION: Duration = Duration::from_millis(100);

pub struct Spinner {
	progress_bar: ProgressBar,
}

impl Spinner {
	pub fn new(message: impl Into<String>, color: &str) -> Self {
		let progress_bar = ProgressBar::new_spinner();
		progress_bar.set_style(
			ProgressStyle::default_spinner()
				.tick_chars(TICK_CHARS)
				.template(&format!("{{spinner:.{color}}} {{msg}}"))
				.unwrap(),
		);
		progress_bar.set_message(message.into());
		progress_bar.enable_steady_tick(TICK_DURATION);
		Self { progress_bar }
	}

	pub fn success(self, message: impl Into<String>) {
		self.progress_bar.finish_with_message(
			format!("{} {}", emoji::SUCCESS, message.into())
				.green()
				.to_string(),
		);
	}

	pub fn error(self, message: impl Into<String>) {
		self.progress_bar.finish_with_message(
			format!("{} {}", emoji::ERROR, message.into())
				.red()
				.to_string(),
		);
	}

	pub fn info(self, message: impl Into<String>) {
		self.progress_bar.finish_with_message(
			format!("{} {}", emoji::INFO, message.into())
				.bright_blue()
				.to_string(),
		);
	}

	pub fn warning(self, message: impl Into<String>) {
		self.progress_bar.finish_with_message(
			format!("{} {}", emoji::WARNING, message.into())
				.yellow()
				.to_string(),
		);
	}

	pub fn done(self, message: impl Into<String>) {
		self.progress_bar.finish_with_message(
			format!("{} {}", emoji::DONE, message.into())
				.green()
				.bold()
				.to_string(),
		);
	}
}
