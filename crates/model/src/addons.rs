//! Built-in addons: device preferences for features Oxide adds on top of Discord.
use serde::{Deserialize, Serialize};

/// Keeps deleted and edited messages visible for this session only; nothing is written to disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageLogger {
	pub enabled: bool,
	/// Keep earlier versions of edited messages.
	pub edit_history: bool,
	/// Alert when a message that mentioned you is deleted.
	pub ghost_pings: bool,
	/// Let bot and app messages disappear normally when deleted.
	pub ignore_bots: bool,
}
impl Default for MessageLogger {
	fn default() -> Self {
		Self {
			enabled: false,
			edit_history: true,
			ghost_pings: true,
			ignore_bots: true,
		}
	}
}
impl MessageLogger {
	pub fn edit_history(self) -> bool {
		self.enabled && self.edit_history
	}
	pub fn ghost_pings(self) -> bool {
		self.enabled && self.ghost_pings
	}
	pub fn ignores(self, author: &crate::User) -> bool {
		self.ignore_bots && (author.webhook || !matches!(author.kind, crate::AccountKind::Human))
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Addons {
	pub message_logger: MessageLogger,
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn older_preferences_gain_defaults_and_unknown_fields_are_ignored() {
		let addons: Addons =
			serde_json::from_str(r#"{"message_logger":{"enabled":true}}"#).unwrap();
		assert!(addons.message_logger.enabled && addons.message_logger.edit_history);
		let addons: Addons = serde_json::from_str(r#"{"future":1}"#).unwrap();
		assert_eq!(addons, Addons::default());
	}
}
