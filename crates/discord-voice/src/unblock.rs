//! Drover-compatible voice unblock. Some networks drop Discord voice when the first datagram a
//! flow carries is the 74-byte IP discovery packet. Like Drover's direct mode, a short decoy
//! sequence precedes it on the same socket: an optional user-supplied datagram (Drover's
//! `drover-packet.bin`), then two one-byte datagrams, then a 50 ms pause. The voice server
//! ignores the decoys; nothing else about the connection changes.
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::net::UdpSocket;

/// Largest datagram payload UDP over IPv4 can carry.
pub const MAX_PACKET: usize = 65_507;
const PAUSE: Duration = Duration::from_millis(50);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VoiceUnblock {
	/// Sent first when present; bounded by [`MAX_PACKET`].
	pub packet: Option<Arc<[u8]>>,
}

static CURRENT: RwLock<Option<VoiceUnblock>> = RwLock::new(None);

/// Applies to every voice and stream connection that starts afterwards. `None` turns it off.
pub fn set(unblock: Option<VoiceUnblock>) {
	let unblock = unblock.map(|mut unblock| {
		unblock.packet = unblock
			.packet
			.filter(|packet| !packet.is_empty() && packet.len() <= MAX_PACKET);
		unblock
	});
	if let Ok(mut current) = CURRENT.write() {
		*current = unblock;
	}
}

fn current() -> Option<VoiceUnblock> {
	CURRENT.read().ok().and_then(|current| current.clone())
}

/// Sends the decoy sequence on a connected socket, before its discovery packet.
/// Send failures are ignored: discovery itself reports a broken path.
pub(crate) async fn before_discovery(socket: &UdpSocket) {
	if let Some(unblock) = current() {
		send_decoys(socket, &unblock).await;
	}
}

async fn send_decoys(socket: &UdpSocket, unblock: &VoiceUnblock) {
	if let Some(packet) = unblock
		.packet
		.as_ref()
		.filter(|packet| !packet.is_empty() && packet.len() <= MAX_PACKET)
	{
		let _ = socket.send(packet).await;
	}
	let _ = socket.send(&[0]).await;
	let _ = socket.send(&[1]).await;
	tokio::time::sleep(PAUSE).await;
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
		for packet in [Arc::from(&b"decoy"[..]), Arc::from(vec![9; MAX_PACKET + 1])] {
			let unblock = VoiceUnblock {
				packet: Some(packet),
			};
			send_decoys(&sender, &unblock).await;
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
