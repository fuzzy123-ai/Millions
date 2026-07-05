use crate::fixtures::DESCRIPTOR_PATHS;
use crate::protocol::PROTOCOL_VERSION;

pub const REPLAY_FORMAT_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvidence {
    pub package_name: &'static str,
    pub package_version: &'static str,
    pub protocol_version: u16,
    pub fixture_descriptor_count: usize,
    pub replay_format_version: u16,
    pub build_id: String,
}

impl BuildEvidence {
    pub fn foundation() -> Self {
        let package_name = env!("CARGO_PKG_NAME");
        let package_version = env!("CARGO_PKG_VERSION");
        let fixture_descriptor_count = DESCRIPTOR_PATHS.len();
        let replay_format_version = REPLAY_FORMAT_VERSION;
        let build_id = format!(
            "{package_name}-{package_version}-protocol_v{PROTOCOL_VERSION}-fixtures_{fixture_descriptor_count}-replay_v{replay_format_version}"
        );

        Self {
            package_name,
            package_version,
            protocol_version: PROTOCOL_VERSION,
            fixture_descriptor_count,
            replay_format_version,
            build_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_evidence_is_stable_and_path_free() {
        let evidence = BuildEvidence::foundation();

        assert_eq!(evidence.package_name, "millions-server");
        assert_eq!(evidence.package_version, "0.1.0");
        assert_eq!(evidence.protocol_version, 0);
        assert_eq!(evidence.fixture_descriptor_count, 3);
        assert_eq!(evidence.replay_format_version, 1);
        assert_eq!(
            evidence.build_id,
            "millions-server-0.1.0-protocol_v0-fixtures_3-replay_v1"
        );
        assert!(!evidence.build_id.contains('\\'));
        assert!(!evidence.build_id.contains('/'));
        assert!(!evidence.build_id.contains(':'));
    }
}
