//! JavaScript bindings for upgrade status.

use napi_derive::napi;
use ws_pkg::UpgradeStatus as WsUpgradeStatus;

/// JavaScript binding for ws_pkg::upgrader::status::UpgradeStatus
#[napi]
pub enum UpgradeStatus {
    /// Dependency is up to date
    UpToDate,
    /// Patch update available (0.0.x)
    PatchAvailable,
    /// Minor update available (0.x.0)
    MinorAvailable,
    /// Major update available (x.0.0)
    MajorAvailable,
    /// Version requirements don't allow update
    Constrained,
    /// Failed to check for updates
    CheckFailed,
}

impl From<WsUpgradeStatus> for UpgradeStatus {
    fn from(status: WsUpgradeStatus) -> Self {
        match status {
            WsUpgradeStatus::UpToDate => Self::UpToDate,
            WsUpgradeStatus::PatchAvailable(_) => Self::PatchAvailable,
            WsUpgradeStatus::MinorAvailable(_) => Self::MinorAvailable,
            WsUpgradeStatus::MajorAvailable(_) => Self::MajorAvailable,
            WsUpgradeStatus::Constrained(_) => Self::Constrained,
            WsUpgradeStatus::CheckFailed(_) => Self::CheckFailed,
        }
    }
}

impl UpgradeStatus {
    pub fn to_ws_upgrade_status(&self) -> ws_pkg::upgrader::UpgradeStatus {
        match self {
            Self::UpToDate => ws_pkg::upgrader::UpgradeStatus::UpToDate,
            Self::PatchAvailable => {
                ws_pkg::upgrader::UpgradeStatus::PatchAvailable("unknown".to_string())
            }
            Self::MinorAvailable => {
                ws_pkg::upgrader::UpgradeStatus::MinorAvailable("unknown".to_string())
            }
            Self::MajorAvailable => {
                ws_pkg::upgrader::UpgradeStatus::MajorAvailable("unknown".to_string())
            }
            Self::Constrained => {
                ws_pkg::upgrader::UpgradeStatus::Constrained("unknown".to_string())
            }
            Self::CheckFailed => {
                ws_pkg::upgrader::UpgradeStatus::CheckFailed("unknown".to_string())
            }
        }
    }
}

#[cfg(test)]
mod status_binding_tests {
    use super::*;

    #[test]
    fn test_upgrade_status_conversion() {
        // Test conversion from WsUpgradeStatus to UpgradeStatus
        assert!(matches!(UpgradeStatus::from(WsUpgradeStatus::UpToDate), UpgradeStatus::UpToDate));
        assert!(matches!(
            UpgradeStatus::from(WsUpgradeStatus::PatchAvailable("1.0.1".to_string())),
            UpgradeStatus::PatchAvailable
        ));
        assert!(matches!(
            UpgradeStatus::from(WsUpgradeStatus::MinorAvailable("1.1.0".to_string())),
            UpgradeStatus::MinorAvailable
        ));
        assert!(matches!(
            UpgradeStatus::from(WsUpgradeStatus::MajorAvailable("2.0.0".to_string())),
            UpgradeStatus::MajorAvailable
        ));
        assert!(matches!(
            UpgradeStatus::from(WsUpgradeStatus::Constrained("2.0.0".to_string())),
            UpgradeStatus::Constrained
        ));
        assert!(matches!(
            UpgradeStatus::from(WsUpgradeStatus::CheckFailed("Error".to_string())),
            UpgradeStatus::CheckFailed
        ));
    }
}
