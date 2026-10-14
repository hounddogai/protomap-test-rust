use super::*;

use acme_kit::text;

/// Where a manual dashboard snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDashboardSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualDashboardSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualDashboardSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual dashboard snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDashboardSnapshot {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub burst: i64,
    pub cipher: String,
    pub approved: bool,
    pub ceiling: f64,
    pub notes: String,
    pub status: ManualDashboardSnapshotStatus,
}

impl ManualDashboardSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDashboardSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDashboardSnapshotStatus) -> bool {
        match self.status {
            ManualDashboardSnapshotStatus::Requested => {
                match next {
                    ManualDashboardSnapshotStatus::Granted | ManualDashboardSnapshotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDashboardSnapshotStatus::Granted => {
                return next == ManualDashboardSnapshotStatus::Revoked
                    || next == ManualDashboardSnapshotStatus::Expired;
            }
            ManualDashboardSnapshotStatus::Revoked => {
                match next {
                    ManualDashboardSnapshotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualDashboardSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual dashboard snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual dashboard snapshot", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ceiling - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDashboardSnapshot {
        ManualDashboardSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            burst: self.burst,
            cipher: self.cipher.clone(),
            approved: self.approved,
            ceiling: self.ceiling,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual dashboard snapshot in status requested.
pub fn new_manual_dashboard_snapshot(code: &str, name: &str) -> ManualDashboardSnapshot {
    ManualDashboardSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        burst: 0,
        cipher: "".to_string(),
        approved: false,
        ceiling: 0.0,
        notes: "".to_string(),
        status: ManualDashboardSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_dashboard_snapshot("code-269", "name-469");
        assert_eq!(item.status, ManualDashboardSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_dashboard_snapshot("code-269", "name-469");
        if item.move_to(ManualDashboardSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDashboardSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_dashboard_snapshot("code-269", "name-469");
        item.status = ManualDashboardSnapshotStatus::Expired;
        assert!(
            item.move_to(ManualDashboardSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
