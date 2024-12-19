use strum::AsRefStr;
use strum::Display as StrumDisplay;
use strum::EnumIter;
use strum::EnumString;

/// The color theme used.
#[derive(
	Default, Debug, Eq, PartialEq, Copy, Clone, StrumDisplay, EnumIter, EnumString, AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum AppEnvironment {
	/// The `local` environment.
	#[default]
	Local,
	/// The `test` environment.
	Test,
	/// The `development` environment.
	Development,
	/// The `production` environment.
	Production,
}
