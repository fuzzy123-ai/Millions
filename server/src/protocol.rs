pub const PROTOCOL_VERSION: u16 = 0;
pub const PROTOCOL_MAGIC: u16 = 0x4D4D;
pub const PROTOCOL_HEADER_LEN: usize = 48;
pub const PROTOCOL_HEADER_LEN_U16: u16 = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    ClientHello = 1,
    ServerHello = 2,
    ClientCommandBatch = 3,
    ServerCommandAck = 4,
    ServerFullSnapshot = 5,
    ServerDeltaSnapshot = 6,
    Ping = 7,
    Disconnect = 8,
}

impl MessageType {
    pub fn from_wire(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::ClientHello),
            2 => Some(Self::ServerHello),
            3 => Some(Self::ClientCommandBatch),
            4 => Some(Self::ServerCommandAck),
            5 => Some(Self::ServerFullSnapshot),
            6 => Some(Self::ServerDeltaSnapshot),
            7 => Some(Self::Ping),
            8 => Some(Self::Disconnect),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    pub message_type: MessageType,
    pub payload_len: u32,
    pub connection_id: u64,
    pub session_id: u64,
    pub client_seq: u32,
    pub server_seq: u32,
    pub ack_seq: u32,
    pub tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDecodeError {
    TooShort,
    BadMagic(u16),
    UnsupportedProtocol(u16),
    UnknownMessageType(u8),
    NonZeroFlags(u8),
    BadHeaderLen(u16),
    PayloadLenMismatch { declared: u32, actual: usize },
}

impl PacketHeader {
    pub fn decode(packet: &[u8]) -> Result<Self, PacketDecodeError> {
        if packet.len() < PROTOCOL_HEADER_LEN {
            return Err(PacketDecodeError::TooShort);
        }

        let magic = read_u16(packet, 0);
        if magic != PROTOCOL_MAGIC {
            return Err(PacketDecodeError::BadMagic(magic));
        }

        let protocol_version = read_u16(packet, 2);
        if protocol_version != PROTOCOL_VERSION {
            return Err(PacketDecodeError::UnsupportedProtocol(protocol_version));
        }

        let raw_message_type = packet[4];
        let message_type = MessageType::from_wire(raw_message_type)
            .ok_or(PacketDecodeError::UnknownMessageType(raw_message_type))?;

        let flags = packet[5];
        if flags != 0 {
            return Err(PacketDecodeError::NonZeroFlags(flags));
        }

        let header_len = read_u16(packet, 6);
        if header_len != PROTOCOL_HEADER_LEN_U16 {
            return Err(PacketDecodeError::BadHeaderLen(header_len));
        }

        let payload_len = read_u32(packet, 8);
        let actual_payload_len = packet.len() - PROTOCOL_HEADER_LEN;
        if payload_len as usize != actual_payload_len {
            return Err(PacketDecodeError::PayloadLenMismatch {
                declared: payload_len,
                actual: actual_payload_len,
            });
        }

        Ok(Self {
            message_type,
            payload_len,
            connection_id: read_u64(packet, 12),
            session_id: read_u64(packet, 20),
            client_seq: read_u32(packet, 28),
            server_seq: read_u32(packet, 32),
            ack_seq: read_u32(packet, 36),
            tick: read_u64(packet, 40),
        })
    }
}

fn read_u16(packet: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([packet[offset], packet[offset + 1]])
}

fn read_u32(packet: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        packet[offset],
        packet[offset + 1],
        packet[offset + 2],
        packet[offset + 3],
    ])
}

fn read_u64(packet: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        packet[offset],
        packet[offset + 1],
        packet[offset + 2],
        packet[offset + 3],
        packet[offset + 4],
        packet[offset + 5],
        packet[offset + 6],
        packet[offset + 7],
    ])
}

pub fn synthetic_session_id(local_identity: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for byte in local_identity.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
