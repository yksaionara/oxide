//! Drover voice unblock, always on and matching Drover's direct mode byte for byte.
//!
//! Some networks (such as in the UAE) drop Discord voice when the first datagram of a flow is
//! the 74-byte IP discovery packet. Drover's `version.dll` hooks the first `WSASendTo` of each
//! UDP socket and, when it carries 74 bytes, first sends:
//! 1. the whole of `drover-packet.bin`, read fresh for each socket, when it exists;
//! 2. one datagram holding the byte `0x00`;
//! 3. one datagram holding the byte `0x01`;
//!
//! then sleeps 50 ms before letting the real packet go. Oxide does the same on every voice
//! and stream socket, right before its discovery packet. A `drover-packet.bin` found on this
//! computer wins; otherwise Oxide sends the copy bundled with it, a 1200-byte QUIC Initial
//! decoy. The voice server ignores the decoys and nothing else about the call changes.
use std::{
	path::PathBuf,
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::net::UdpSocket;

/// Largest datagram payload UDP over IPv4 can carry.
pub const MAX_PACKET: usize = 65_507;
/// Drover's own pause before the real packet.
const PAUSE: Duration = Duration::from_millis(50);
/// The decoy shipped with Oxide, used when this computer has no `drover-packet.bin`.
pub const BUNDLED_PACKET: &[u8] = include_bytes!("../assets/drover-packet.bin");

static LOCAL_PACKET: RwLock<Option<Arc<PathBuf>>> = RwLock::new(None);

/// A `drover-packet.bin` found on this computer, read again for every new socket like Drover.
/// `None` uses the bundled packet.
pub fn set_local_packet(path: Option<PathBuf>) {
	if let Ok(mut current) = LOCAL_PACKET.write() {
		*current = path.map(Arc::new);
	}
}

/// The decoy for one new socket: the local file when it is readable and bounded, otherwise
/// the bundled one. A missing or broken local file never stops a call.
async fn packet() -> Arc<[u8]> {
	let path = LOCAL_PACKET.read().ok().and_then(|path| path.clone());
	if let Some(path) = path
		&& let Ok(metadata) = tokio::fs::metadata(path.as_ref()).await
		&& (1..=MAX_PACKET as u64).contains(&metadata.len())
		&& let Ok(bytes) = tokio::fs::read(path.as_ref()).await
		&& (1..=MAX_PACKET).contains(&bytes.len())
	{
		return Arc::from(bytes);
	}
	Arc::from(BUNDLED_PACKET)
}

/// Sends Drover's decoy sequence on a connected socket, before its discovery packet.
/// Send failures are ignored: discovery itself reports a broken path.
pub(crate) async fn before_discovery(socket: &UdpSocket) {
	send_decoys(socket, &packet().await).await;
}

async fn send_decoys(socket: &UdpSocket, packet: &[u8]) {
	if (1..=MAX_PACKET).contains(&packet.len()) {
		let _ = socket.send(packet).await;
	}
	let _ = socket.send(&[0x00]).await;
	let _ = socket.send(&[0x01]).await;
	tokio::time::sleep(PAUSE).await;
}

/// Test voice servers: asserts Drover's decoys arrive in order on a fresh socket, then
/// returns the discovery packet's length and sender.
#[cfg(test)]
pub(crate) async fn expect_decoys_then_discovery(
	udp: &UdpSocket,
	buffer: &mut [u8],
) -> (usize, std::net::SocketAddr) {
	assert!(buffer.len() > BUNDLED_PACKET.len());
	for expected in [BUNDLED_PACKET, &[0x00][..], &[0x01][..]] {
		let (length, _) = udp.recv_from(buffer).await.unwrap();
		assert_eq!(&buffer[..length], expected, "Drover decoy out of order");
	}
	udp.recv_from(buffer).await.unwrap()
}

#[cfg(test)]
mod tests {
	use super::*;
	#[tokio::test]
	async fn decoys_precede_discovery_in_drover_order() {
		let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
		let sender = UdpSocket::bind("127.0.0.1:0").await.unwrap();
		sender
			.connect(receiver.local_addr().unwrap())
			.await
			.unwrap();
		for packet in [&b"decoy"[..], &vec![9; MAX_PACKET + 1][..]] {
			send_decoys(&sender, packet).await;
			sender.send(&[7; 74]).await.unwrap();
		}
		let mut buffer = vec![0; MAX_PACKET + 8];
		let mut seen = Vec::new();
		for _ in 0..7 {
			let length = receiver.recv(&mut buffer).await.unwrap();
			seen.push(buffer[..length].to_vec());
		}
		// An oversized packet is skipped; the one-byte decoys still go first.
		assert_eq!(
			seen,
			vec![
				b"decoy".to_vec(),
				vec![0],
				vec![1],
				vec![7; 74],
				vec![0],
				vec![1],
				vec![7; 74]
			]
		);
	}
}
