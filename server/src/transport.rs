use crate::simulation::PlayerSessionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Loopback,
    Udp,
    MockSteam,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportEnvelope {
    pub connection_id: ConnectionId,
    pub session_id: Option<PlayerSessionId>,
    pub kind: TransportKind,
}
