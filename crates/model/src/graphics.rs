//! Device-local GPU selection; independent of Discord accounts.

/// Which GPU Oxide renders on, mirroring the three choices desktop platforms already offer.
///
/// A preference only orders the adapters that can actually present to the window, so it can
/// never select a device the compositor refuses to read from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GpuPreference {
	/// Oxide picks: presentable adapters first, then the discrete GPU that usually drives
	/// the display. Honors `WGPU_POWER_PREF` for diagnostics.
	#[default]
	Automatic,
	/// Prefer the discrete GPU even when an integrated one could present.
	HighPerformance,
	/// Prefer the integrated GPU to save battery and fan noise on laptops.
	PowerSaving,
}
impl GpuPreference {
	pub const ALL: [Self; 3] = [Self::Automatic, Self::HighPerformance, Self::PowerSaving];
	pub fn label(self) -> &'static str {
		match self {
			Self::Automatic => "Automatic",
			Self::HighPerformance => "High performance",
			Self::PowerSaving => "Power saving",
		}
	}
	pub fn description(self) -> &'static str {
		match self {
			Self::Automatic => "Let Oxide choose the GPU that can draw this window.",
			Self::HighPerformance => "Use the discrete graphics card when one is available.",
			Self::PowerSaving => "Use integrated graphics to save battery.",
		}
	}
	/// Stable storage key; unknown keys from a newer build fall back to [`Self::Automatic`].
	fn key(self) -> &'static str {
		match self {
			Self::Automatic => "automatic",
			Self::HighPerformance => "high-performance",
			Self::PowerSaving => "power-saving",
		}
	}
}
impl From<String> for GpuPreference {
	fn from(value: String) -> Self {
		Self::ALL
			.into_iter()
			.find(|candidate| candidate.key() == value)
			.unwrap_or_default()
	}
}
impl From<GpuPreference> for String {
	fn from(value: GpuPreference) -> Self {
		value.key().to_owned()
	}
}
impl serde::Serialize for GpuPreference {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.key())
	}
}
impl<'de> serde::Deserialize<'de> for GpuPreference {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		// Tolerate a value written by a newer build rather than rejecting every preference.
		Ok(Self::from(String::deserialize(deserializer)?))
	}
}

/// Device-local rendering API preference. Unknown keys from newer builds use Automatic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GraphicsApi {
	Vulkan,
	#[serde(rename = "directx12")]
	DirectX12,
	Metal,
	OpenGl,
	#[default]
	#[serde(other)]
	Automatic,
}
impl GraphicsApi {
	pub const ALL: [Self; 5] = [
		Self::Automatic,
		Self::Vulkan,
		Self::DirectX12,
		Self::Metal,
		Self::OpenGl,
	];
	pub fn label(self) -> &'static str {
		match self {
			Self::Automatic => "Automatic",
			Self::Vulkan => "Vulkan",
			Self::DirectX12 => "DirectX 12",
			Self::Metal => "Metal",
			Self::OpenGl => "OpenGL",
		}
	}
}

/// Most frames per second Oxide draws for pointer motion, scrolling and animations. Immediate
/// mode redraws the whole window each frame, so a 240 or 360 Hz display otherwise multiplies
/// CPU and GPU use for no visible gain in a chat client. Unknown keys from newer builds use 60.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrameRateLimit {
	#[serde(rename = "30")]
	Fps30,
	#[serde(rename = "120")]
	Fps120,
	#[serde(rename = "144")]
	Fps144,
	#[serde(rename = "240")]
	Fps240,
	/// Every display refresh, as before the limit existed.
	#[serde(rename = "display")]
	Display,
	#[default]
	#[serde(rename = "60", other)]
	Fps60,
}
impl FrameRateLimit {
	pub const ALL: [Self; 6] = [
		Self::Fps30,
		Self::Fps60,
		Self::Fps120,
		Self::Fps144,
		Self::Fps240,
		Self::Display,
	];
	/// Frames per second, or `None` to follow the display.
	pub fn fps(self) -> Option<u32> {
		match self {
			Self::Fps30 => Some(30),
			Self::Fps60 => Some(60),
			Self::Fps120 => Some(120),
			Self::Fps144 => Some(144),
			Self::Fps240 => Some(240),
			Self::Display => None,
		}
	}
	/// Shortest time between two frames; zero follows the display.
	pub fn interval(self) -> std::time::Duration {
		self.fps().map_or(std::time::Duration::ZERO, |fps| {
			std::time::Duration::from_secs(1) / fps
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn round_trips_through_storage_keys() {
		{
			for preference in GpuPreference::ALL {
				let key = String::from(preference);
				assert_eq!(GpuPreference::from(key), preference);
			}
		}
		{
			assert_eq!(GpuPreference::default(), GpuPreference::Automatic);
			assert_eq!(
				GpuPreference::from("quantum-gpu".to_owned()),
				GpuPreference::Automatic
			);
			assert_eq!(GpuPreference::from(String::new()), GpuPreference::Automatic);
		}
	}
}
