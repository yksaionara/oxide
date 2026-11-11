# Voice unblock asset

`drover-packet.bin` is the decoy datagram Oxide sends before voice and stream UDP discovery when
this computer has no `drover-packet.bin` of its own (see `src/unblock.rs`). It is a 1200-byte
QUIC Initial packet, the kind DPI-bypass tools ship as a fake handshake, copied from the owner's
working Drover (direct mode) setup. It carries no account, device or personal data and is sent
unchanged, exactly as Drover sends its file.
