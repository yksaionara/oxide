//! Message Logger session state. Deleted and edited bodies stay in the bounded timeline;
//! this only remembers recent mentions of the owner, so a deleted one can be reported as a
//! ghost ping. Nothing here is persisted.
use model::Id;
use std::collections::VecDeque;

pub(crate) const MAX_MENTIONS: usize = 64;
pub(crate) const MAX_GHOST_PINGS: usize = 16;

pub(crate) struct Mention {
	pub channel: Id,
	pub message: Id,
	pub sender: String,
	pub preview: String,
	pub avatar_key: String,
}

/// A deleted message that had mentioned the owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GhostPing {
	pub channel: Id,
	pub sender: String,
	pub preview: String,
}

#[derive(Default)]
pub struct Logger {
	pub(crate) mentions: VecDeque<Mention>,
	pub(crate) ghost_pings: VecDeque<GhostPing>,
}
impl Logger {
	pub(crate) fn remember(&mut self, mention: Mention) {
		if self
			.mentions
			.iter()
			.any(|known| known.message == mention.message)
		{
			return;
		}
		if self.mentions.len() >= MAX_MENTIONS {
			self.mentions.pop_front();
		}
		self.mentions.push_back(mention);
	}
	pub(crate) fn take_deleted(&mut self, channel: Id, message: Id) -> Option<Mention> {
		let index = self
			.mentions
			.iter()
			.position(|known| known.channel == channel && known.message == message)?;
		self.mentions.remove(index)
	}
	pub(crate) fn report(&mut self, ping: GhostPing) {
		if self.ghost_pings.len() >= MAX_GHOST_PINGS {
			self.ghost_pings.pop_front();
		}
		self.ghost_pings.push_back(ping);
	}
	pub fn clear(&mut self) {
		self.mentions.clear();
		self.ghost_pings.clear();
	}
}
