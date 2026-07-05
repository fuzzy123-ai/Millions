use crate::protocol::synthetic_session_id;
use crate::simulation::PlayerSessionId;
use crate::transport::{ConnectionId, TransportEnvelope, TransportKind};

pub const STEAM_BRIDGE_SCHEMA: &str = "steam_bridge_v0";
pub const STEAM_BRIDGE_PROTOCOL: &str = "protocol_v0";
pub const LOCAL_MOCK_IDENTITY_MODE: &str = "local_mock";
pub const LOCAL_DIRECT_SERVER_MODE: &str = "local_direct";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamBridgeHandoff {
    pub schema: String,
    pub identity_mode: String,
    pub lobby_id: String,
    pub lobby_state: String,
    pub protocol: String,
    pub server_mode: String,
    pub server_endpoint: String,
    pub endpoint_epoch: u64,
    pub ready_epoch: u64,
    pub build_id: String,
    pub player_display_name: String,
    pub player_session_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamBridgeError {
    BadSchema,
    UnsupportedIdentityMode,
    LobbyNotReady,
    BadProtocol,
    UnsupportedServerMode,
    EmptyEndpoint,
    EmptyPlayerSession,
    NonMonotonicEpoch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptedSteamBridgeSession {
    pub connection_id: ConnectionId,
    pub player_session_id: PlayerSessionId,
    pub transport: TransportEnvelope,
}

pub fn accept_local_mock_handoff(
    handoff: &SteamBridgeHandoff,
    connection_id: ConnectionId,
) -> Result<AcceptedSteamBridgeSession, SteamBridgeError> {
    validate_local_mock_handoff(handoff)?;

    let session_id = PlayerSessionId(synthetic_session_id(&format!(
        "{}:{}",
        handoff.lobby_id, handoff.player_display_name
    )));
    let transport = TransportEnvelope {
        connection_id,
        session_id: Some(session_id),
        kind: TransportKind::MockSteam,
    };

    Ok(AcceptedSteamBridgeSession {
        connection_id,
        player_session_id: session_id,
        transport,
    })
}

pub fn validate_local_mock_handoff(handoff: &SteamBridgeHandoff) -> Result<(), SteamBridgeError> {
    if handoff.schema != STEAM_BRIDGE_SCHEMA {
        return Err(SteamBridgeError::BadSchema);
    }
    if handoff.identity_mode != LOCAL_MOCK_IDENTITY_MODE {
        return Err(SteamBridgeError::UnsupportedIdentityMode);
    }
    if handoff.lobby_state != "ready" && handoff.lobby_state != "launching" {
        return Err(SteamBridgeError::LobbyNotReady);
    }
    if handoff.protocol != STEAM_BRIDGE_PROTOCOL {
        return Err(SteamBridgeError::BadProtocol);
    }
    if handoff.server_mode != LOCAL_DIRECT_SERVER_MODE {
        return Err(SteamBridgeError::UnsupportedServerMode);
    }
    if handoff.server_endpoint.trim().is_empty() {
        return Err(SteamBridgeError::EmptyEndpoint);
    }
    if handoff.player_session_id.trim().is_empty() {
        return Err(SteamBridgeError::EmptyPlayerSession);
    }
    if handoff.endpoint_epoch == 0 || handoff.ready_epoch == 0 {
        return Err(SteamBridgeError::NonMonotonicEpoch);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_handoff() -> SteamBridgeHandoff {
        SteamBridgeHandoff {
            schema: STEAM_BRIDGE_SCHEMA.to_string(),
            identity_mode: LOCAL_MOCK_IDENTITY_MODE.to_string(),
            lobby_id: "local-abc123".to_string(),
            lobby_state: "ready".to_string(),
            protocol: STEAM_BRIDGE_PROTOCOL.to_string(),
            server_mode: LOCAL_DIRECT_SERVER_MODE.to_string(),
            server_endpoint: "127.0.0.1:7777".to_string(),
            endpoint_epoch: 1,
            ready_epoch: 1,
            build_id: "local-uncommitted".to_string(),
            player_display_name: "Player 1".to_string(),
            player_session_id: "local-session-local-abc123-player-1".to_string(),
        }
    }

    #[test]
    fn local_mock_handoff_accepts_redacted_facade_shape() {
        let handoff = valid_handoff();
        let accepted = accept_local_mock_handoff(&handoff, ConnectionId(77))
            .expect("local mock handoff accepts");

        assert_eq!(accepted.connection_id, ConnectionId(77));
        assert_ne!(accepted.player_session_id, PlayerSessionId(0));
        assert_eq!(accepted.transport.kind, TransportKind::MockSteam);
        assert_eq!(
            accepted.transport.session_id,
            Some(accepted.player_session_id)
        );
    }

    #[test]
    fn local_mock_handoff_rejects_live_or_unready_shapes() {
        let mut handoff = valid_handoff();
        handoff.identity_mode = "steam".to_string();
        assert_eq!(
            validate_local_mock_handoff(&handoff),
            Err(SteamBridgeError::UnsupportedIdentityMode)
        );

        handoff = valid_handoff();
        handoff.lobby_state = "ready_pending".to_string();
        assert_eq!(
            validate_local_mock_handoff(&handoff),
            Err(SteamBridgeError::LobbyNotReady)
        );

        handoff = valid_handoff();
        handoff.endpoint_epoch = 0;
        assert_eq!(
            validate_local_mock_handoff(&handoff),
            Err(SteamBridgeError::NonMonotonicEpoch)
        );
    }
}
