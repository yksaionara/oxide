//! Features built into Oxide itself: the Message Logger card shown first on the
//! Extensions page, and the Drover voice unblock status in Voice settings.
use crate::{MessagingUi, design};

/// Built-in Message Logger settings, listed above community extensions.
pub(crate) fn message_logger_card(ui: &mut egui::Ui, logger: &mut model::addons::MessageLogger) {
	design::group(ui, "addons-message-logger", |ui| {
		design::switch(
			ui,
			"addons-message-logger",
			Some("addons-message-logger-description"),
			&mut logger.enabled,
		);
		ui.add_enabled_ui(logger.enabled, |ui| {
			design::card_divider(ui);
			design::switch(
				ui,
				"addons-logger-edit-history",
				Some("addons-logger-edit-history-description"),
				&mut logger.edit_history,
			);
			design::switch(
				ui,
				"addons-logger-ghost-pings",
				Some("addons-logger-ghost-pings-description"),
				&mut logger.ghost_pings,
			);
			design::switch(
				ui,
				"addons-logger-ignore-bots",
				Some("addons-logger-ignore-bots-description"),
				&mut logger.ignore_bots,
			);
		});
	});
}

impl MessagingUi {
	/// Drover voice unblock is always on; this card only reports which packet it uses.
	pub(super) fn voice_unblock_card(&mut self, ui: &mut egui::Ui) {
		design::group(ui, "addons-voice-unblock", |ui| {
			design::row(
				ui,
				"addons-voice-unblock-status",
				Some(&self.voice_unblock_status.clone()),
				|ui| {
					if design::button(
						ui,
						&crate::i18n::translate("addons-voice-unblock-rescan"),
						design::ButtonKind::Outline,
					)
					.clicked()
					{
						self.voice_unblock_rescan = true;
					}
				},
			);
			design::hint(ui, &crate::i18n::translate("addons-voice-unblock-hint"));
		});
	}
}
