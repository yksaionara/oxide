//! Settings > Addons: features built into Oxide itself, as opposed to community extensions.
use crate::{MessagingUi, design};

impl MessagingUi {
	pub(super) fn addons_page(&mut self, ui: &mut egui::Ui) {
		design::group(ui, "addons-message-logger", |ui| {
			let logger = &mut self.addons.message_logger;
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
		design::group(ui, "addons-voice-unblock", |ui| {
			design::switch(
				ui,
				"addons-voice-unblock",
				Some("addons-voice-unblock-description"),
				&mut self.addons.voice_unblock,
			);
			if self.addons.voice_unblock {
				design::card_divider(ui);
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
			}
		});
	}
}
