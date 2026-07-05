use crate::simulation::{PlayerSessionId, Tick};
use crate::transport::ConnectionId;

pub const DEFAULT_RECONNECT_GRACE_TICKS: u64 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionConnectionState {
    Connected,
    GracePeriod,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebindResult {
    Rebound {
        previous_connection_id: ConnectionId,
    },
    SameConnection,
    NotInGracePeriod,
    GraceExpired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotResumeAction {
    SendFullSnapshot,
    ResumeDeltaStream { baseline_snapshot_id: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionRebindState {
    pub player_session_id: PlayerSessionId,
    pub active_connection_id: ConnectionId,
    pub previous_connection_id: Option<ConnectionId>,
    pub state: SessionConnectionState,
    pub last_seen_tick: Tick,
    pub grace_expires_at_tick: Tick,
}

impl SessionRebindState {
    pub fn connected(
        player_session_id: PlayerSessionId,
        connection_id: ConnectionId,
        now: Tick,
        grace_ticks: u64,
    ) -> Self {
        Self {
            player_session_id,
            active_connection_id: connection_id,
            previous_connection_id: None,
            state: SessionConnectionState::Connected,
            last_seen_tick: now,
            grace_expires_at_tick: Tick(now.0.saturating_add(grace_ticks)),
        }
    }

    pub fn enter_grace_period(&mut self, now: Tick, grace_ticks: u64) {
        if self.state == SessionConnectionState::Expired {
            return;
        }

        self.state = SessionConnectionState::GracePeriod;
        self.last_seen_tick = now;
        self.grace_expires_at_tick = Tick(now.0.saturating_add(grace_ticks));
    }

    pub fn rebind(&mut self, next_connection_id: ConnectionId, now: Tick) -> RebindResult {
        self.expire_if_due(now);

        if self.active_connection_id == next_connection_id {
            return RebindResult::SameConnection;
        }

        match self.state {
            SessionConnectionState::GracePeriod => {
                let previous_connection_id = self.active_connection_id;
                self.previous_connection_id = Some(previous_connection_id);
                self.active_connection_id = next_connection_id;
                self.state = SessionConnectionState::Connected;
                self.last_seen_tick = now;
                RebindResult::Rebound {
                    previous_connection_id,
                }
            }
            SessionConnectionState::Connected => RebindResult::NotInGracePeriod,
            SessionConnectionState::Expired => RebindResult::GraceExpired,
        }
    }

    pub fn expire_if_due(&mut self, now: Tick) {
        if self.state == SessionConnectionState::GracePeriod && now.0 > self.grace_expires_at_tick.0
        {
            self.state = SessionConnectionState::Expired;
        }
    }

    pub fn accepts_connection(&self, connection_id: ConnectionId) -> bool {
        self.state == SessionConnectionState::Connected
            && self.active_connection_id == connection_id
    }

    pub fn needs_full_snapshot(&self, connection_id: ConnectionId) -> bool {
        self.accepts_connection(connection_id) && self.previous_connection_id.is_some()
    }

    pub fn snapshot_resume_action(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SnapshotResumeAction> {
        if self.needs_full_snapshot(connection_id) {
            Some(SnapshotResumeAction::SendFullSnapshot)
        } else {
            None
        }
    }

    pub fn mark_full_snapshot_sent(
        &mut self,
        connection_id: ConnectionId,
        snapshot_id: u64,
    ) -> Option<SnapshotResumeAction> {
        if !self.needs_full_snapshot(connection_id) {
            return None;
        }

        self.previous_connection_id = None;
        Some(SnapshotResumeAction::ResumeDeltaStream {
            baseline_snapshot_id: snapshot_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: PlayerSessionId = PlayerSessionId(42);
    const OLD_CONNECTION: ConnectionId = ConnectionId(100);
    const NEW_CONNECTION: ConnectionId = ConnectionId(200);

    #[test]
    fn session_starts_connected_with_grace_deadline() {
        let state = SessionRebindState::connected(SESSION, OLD_CONNECTION, Tick(10), 5);

        assert_eq!(state.player_session_id, SESSION);
        assert_eq!(state.active_connection_id, OLD_CONNECTION);
        assert_eq!(state.state, SessionConnectionState::Connected);
        assert_eq!(state.grace_expires_at_tick, Tick(15));
        assert!(state.accepts_connection(OLD_CONNECTION));
    }

    #[test]
    fn rebind_changes_connection_inside_grace_period() {
        let mut state = SessionRebindState::connected(
            SESSION,
            OLD_CONNECTION,
            Tick(10),
            DEFAULT_RECONNECT_GRACE_TICKS,
        );
        state.enter_grace_period(Tick(20), 5);

        let result = state.rebind(NEW_CONNECTION, Tick(24));

        assert_eq!(
            result,
            RebindResult::Rebound {
                previous_connection_id: OLD_CONNECTION,
            }
        );
        assert_eq!(state.state, SessionConnectionState::Connected);
        assert_eq!(state.previous_connection_id, Some(OLD_CONNECTION));
        assert!(state.accepts_connection(NEW_CONNECTION));
        assert!(!state.accepts_connection(OLD_CONNECTION));
        assert!(state.needs_full_snapshot(NEW_CONNECTION));
    }

    #[test]
    fn full_snapshot_clears_reconnect_delta_resume_gate() {
        let mut state = SessionRebindState::connected(SESSION, OLD_CONNECTION, Tick(10), 5);
        state.enter_grace_period(Tick(20), 5);
        assert_eq!(
            state.rebind(NEW_CONNECTION, Tick(24)),
            RebindResult::Rebound {
                previous_connection_id: OLD_CONNECTION,
            }
        );

        assert_eq!(
            state.snapshot_resume_action(NEW_CONNECTION),
            Some(SnapshotResumeAction::SendFullSnapshot)
        );
        assert_eq!(
            state.mark_full_snapshot_sent(NEW_CONNECTION, 900),
            Some(SnapshotResumeAction::ResumeDeltaStream {
                baseline_snapshot_id: 900,
            })
        );
        assert_eq!(state.snapshot_resume_action(NEW_CONNECTION), None);
        assert!(!state.needs_full_snapshot(NEW_CONNECTION));
    }

    #[test]
    fn rebind_is_rejected_after_grace_expires() {
        let mut state = SessionRebindState::connected(SESSION, OLD_CONNECTION, Tick(10), 5);
        state.enter_grace_period(Tick(20), 5);

        let result = state.rebind(NEW_CONNECTION, Tick(26));

        assert_eq!(result, RebindResult::GraceExpired);
        assert_eq!(state.state, SessionConnectionState::Expired);
        assert_eq!(state.active_connection_id, OLD_CONNECTION);
        assert!(!state.accepts_connection(NEW_CONNECTION));
    }

    #[test]
    fn new_connection_is_not_accepted_before_grace_period() {
        let mut state = SessionRebindState::connected(SESSION, OLD_CONNECTION, Tick(10), 5);

        let result = state.rebind(NEW_CONNECTION, Tick(11));

        assert_eq!(result, RebindResult::NotInGracePeriod);
        assert!(state.accepts_connection(OLD_CONNECTION));
        assert!(!state.accepts_connection(NEW_CONNECTION));
    }
}
