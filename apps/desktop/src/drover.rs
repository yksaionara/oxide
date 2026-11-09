//! Voice unblock addon: finds an existing Drover setup and applies its voice behavior natively.
//! Nothing is injected into Discord or Oxide. Lookup runs on a background thread, only when the
//! addon is enabled, and reads at most two small files.
use std::{
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};

const PACKET_FILE: &str = "drover-packet.bin";
const OPTIONS_FILE: &str = "drover.ini";
const MAX_OPTIONS: u64 = 16 * 1024;

#[derive(Default)]
pub struct Drover {
	applied: Option<bool>,
	status: Arc<Mutex<Option<String>>>,
}

impl Drover {
	/// Applies the addon when it is turned on or off and publishes the lookup result.
	pub fn sync(&mut self, messaging: &mut ui::MessagingUi) {
		let enabled = messaging.addons.voice_unblock;
		if self.applied != Some(enabled) || messaging.voice_unblock_rescan {
			messaging.voice_unblock_rescan = false;
			self.applied = Some(enabled);
			if enabled {
				let status = self.status.clone();
				if let Ok(mut status) = status.lock() {
					*status = Some("Looking for Drover…".into());
				}
				let spawned = std::thread::Builder::new()
					.name("oxide-voice-unblock".into())
					.spawn(move || {
						let found = find();
						discord_voice::unblock::set(Some(discord_voice::unblock::VoiceUnblock {
							packet: found
								.packet
								.as_ref()
								.map(|(_, bytes)| Arc::from(&bytes[..])),
						}));
						if let Ok(mut status) = status.lock() {
							*status = Some(found.describe());
						}
					});
				if spawned.is_err() {
					discord_voice::unblock::set(Some(Default::default()));
					messaging.voice_unblock_status = "On. Decoy packets only.".into();
				}
			} else {
				discord_voice::unblock::set(None);
				messaging.voice_unblock_status = "Off".into();
				if let Ok(mut status) = self.status.lock() {
					*status = None;
				}
			}
		}
		if let Ok(mut status) = self.status.try_lock()
			&& let Some(status) = status.take()
		{
			messaging.voice_unblock_status = status;
		}
	}
}

#[derive(Default)]
struct Found {
	packet: Option<(PathBuf, Vec<u8>)>,
	proxy: Option<String>,
}
impl Found {
	fn describe(&self) -> String {
		let mut text = match &self.packet {
			Some((path, bytes)) => format!(
				"On. Using Drover's packet ({} bytes) from {}.",
				bytes.len(),
				path.parent().and_then(Path::file_name).map_or_else(
					|| path.display().to_string(),
					|name| name.to_string_lossy().into_owned()
				)
			),
			None => "On. No drover-packet.bin found; sending decoy packets only.".into(),
		};
		if let Some(proxy) = &self.proxy {
			text.push_str(&format!(
				" Drover's proxy setting ({proxy}) is not used: Oxide connects directly."
			));
		}
		text
	}
}

/// Oxide's own folder first, so a packet placed beside oxide.exe wins; then Discord's newest
/// app folder that contains Drover files.
fn find() -> Found {
	let mut folders = Vec::new();
	if let Some(folder) = std::env::current_exe()
		.ok()
		.and_then(|exe| exe.parent().map(Path::to_path_buf))
	{
		folders.push(folder);
	}
	if let Ok(data) = local_store::data_dir() {
		folders.push(data);
	}
	folders.extend(discord_app_folders());
	let mut found = Found::default();
	for folder in folders {
		if found.packet.is_none()
			&& let Some(bytes) = read_bounded(
				&folder.join(PACKET_FILE),
				discord_voice::unblock::MAX_PACKET as u64,
			) && !bytes.is_empty()
		{
			found.packet = Some((folder.join(PACKET_FILE), bytes));
		}
		if found.proxy.is_none()
			&& let Some(options) = read_bounded(&folder.join(OPTIONS_FILE), MAX_OPTIONS)
		{
			found.proxy = proxy(&String::from_utf8_lossy(&options)).map(redact);
		}
	}
	found
}

fn read_bounded(path: &Path, limit: u64) -> Option<Vec<u8>> {
	use std::io::Read as _;
	let file = std::fs::File::open(path).ok()?;
	if file.metadata().ok()?.len() > limit {
		return None;
	}
	let mut bytes = Vec::new();
	file.take(limit + 1).read_to_end(&mut bytes).ok()?;
	(bytes.len() as u64 <= limit).then_some(bytes)
}

/// Discord, PTB and Canary `app-x.y.z` folders, newest version first.
fn discord_app_folders() -> Vec<PathBuf> {
	let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
		return Vec::new();
	};
	let mut folders = Vec::new();
	for product in ["Discord", "DiscordPTB", "DiscordCanary"] {
		let Ok(entries) = std::fs::read_dir(local.join(product)) else {
			continue;
		};
		let mut versions = entries
			.flatten()
			.take(64)
			.filter_map(|entry| {
				let name = entry.file_name().into_string().ok()?;
				let version = name
					.strip_prefix("app-")?
					.split('.')
					.map(|part| part.parse::<u32>().ok())
					.collect::<Option<Vec<_>>>()?;
				Some((version, entry.path()))
			})
			.collect::<Vec<_>>();
		versions.sort_by(|a, b| b.0.cmp(&a.0));
		folders.extend(versions.into_iter().map(|(_, path)| path));
	}
	folders
}

/// Drover's `[drover] proxy = …` value; empty means direct mode.
fn proxy(options: &str) -> Option<String> {
	let mut in_section = false;
	for line in options.lines() {
		let line = line.trim();
		if line.starts_with(';') || line.starts_with('#') || line.is_empty() {
			continue;
		}
		if line.starts_with('[') {
			in_section = line.eq_ignore_ascii_case("[drover]");
			continue;
		}
		if in_section
			&& let Some((key, value)) = line.split_once('=')
			&& key.trim().eq_ignore_ascii_case("proxy")
		{
			let value = value.trim();
			return (!value.is_empty()).then(|| value.chars().take(256).collect());
		}
	}
	None
}

/// Never show proxy credentials.
fn redact(proxy: String) -> String {
	match proxy.rsplit_once('@') {
		Some((before, host)) => {
			let scheme = before.split_once("://").map_or("", |(scheme, _)| scheme);
			if scheme.is_empty() {
				format!("***@{host}")
			} else {
				format!("{scheme}://***@{host}")
			}
		}
		None => proxy,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn reads_drover_proxy_like_drover_and_hides_credentials() {
		assert_eq!(proxy("[drover]\nproxy =\n"), None);
		assert_eq!(
			proxy(
				"; comment\n[other]\nproxy = x:1\n[Drover]\n proxy = socks5://u:p@1.2.3.4:1080 \n"
			),
			Some("socks5://u:p@1.2.3.4:1080".into())
		);
		assert_eq!(
			redact("socks5://u:p@1.2.3.4:1080".into()),
			"socks5://***@1.2.3.4:1080"
		);
		assert_eq!(redact("u:p@host:80".into()), "***@host:80");
		assert_eq!(
			redact("http://127.0.0.1:1080".into()),
			"http://127.0.0.1:1080"
		);
	}
}
