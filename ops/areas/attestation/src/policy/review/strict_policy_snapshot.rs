use super::*;

use acme_kit::text;

/// Where a strict policy snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPolicySnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictPolicySnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for StrictPolicySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict policy snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySnapshot {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub margin: f64,
    pub priority: i64,
    pub cipher: String,
    pub expires_at: i64,
    pub approved: bool,
    pub audience: String,
    pub status: StrictPolicySnapshotStatus,
}

impl StrictPolicySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPolicySnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPolicySnapshotStatus) -> bool {
        match self.status {
            StrictPolicySnapshotStatus::Open => {
                match next {
                    StrictPolicySnapshotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPolicySnapshotStatus::InReview => {
                return next == StrictPolicySnapshotStatus::Reopened || next == StrictPolicySnapshotStatus::Archived;
            }
            StrictPolicySnapshotStatus::Resolved => {
                return next == StrictPolicySnapshotStatus::Reopened;
            }
            StrictPolicySnapshotStatus::Reopened => {
                match next {
                    StrictPolicySnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictPolicySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict policy snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.port as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPolicySnapshot {
        StrictPolicySnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            margin: self.margin,
            priority: self.priority,
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            approved: self.approved,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict policy snapshot in status open.
pub fn new_strict_policy_snapshot(code: &str, name: &str) -> StrictPolicySnapshot {
    StrictPolicySnapshot {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        margin: 0.0,
        priority: 0,
        cipher: "".to_string(),
        expires_at: 0,
        approved: false,
        audience: "".to_string(),
        status: StrictPolicySnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_policy_snapshot("code-165", "name-95");
        assert_eq!(item.status, StrictPolicySnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_strict_policy_snapshot("code-165", "name-95");
        if item.move_to(StrictPolicySnapshotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPolicySnapshotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_policy_snapshot("code-165", "name-95");
        item.status = StrictPolicySnapshotStatus::Archived;
        assert!(
            item.move_to(StrictPolicySnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
