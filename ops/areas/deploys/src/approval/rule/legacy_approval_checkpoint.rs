use super::*;

use acme_kit::text;

/// Where a legacy approval checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyApprovalCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyApprovalCheckpointStatus {
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

impl std::fmt::Display for LegacyApprovalCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy approval checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyApprovalCheckpoint {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub cost: f64,
    pub checksum: String,
    pub burst: i64,
    pub updated_at: i64,
    pub fingerprint: String,
    pub attempts: i64,
    pub status: LegacyApprovalCheckpointStatus,
}

impl LegacyApprovalCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyApprovalCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyApprovalCheckpointStatus) -> bool {
        match self.status {
            LegacyApprovalCheckpointStatus::Open => {
                return next == LegacyApprovalCheckpointStatus::Resolved;
            }
            LegacyApprovalCheckpointStatus::InReview => {
                match next {
                    LegacyApprovalCheckpointStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyApprovalCheckpointStatus::Resolved => {
                return next == LegacyApprovalCheckpointStatus::Archived;
            }
            LegacyApprovalCheckpointStatus::Reopened => {
                match next {
                    LegacyApprovalCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyApprovalCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy approval checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.cost + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyApprovalCheckpoint {
        LegacyApprovalCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            cost: self.cost,
            checksum: self.checksum.clone(),
            burst: self.burst,
            updated_at: self.updated_at,
            fingerprint: self.fingerprint.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy approval checkpoint in status open.
pub fn new_legacy_approval_checkpoint(code: &str, name: &str) -> LegacyApprovalCheckpoint {
    LegacyApprovalCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        cost: 0.0,
        checksum: "".to_string(),
        burst: 0,
        updated_at: 0,
        fingerprint: "".to_string(),
        attempts: 0,
        status: LegacyApprovalCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_approval_checkpoint("code-734", "name-892");
        assert_eq!(item.status, LegacyApprovalCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_legacy_approval_checkpoint("code-734", "name-892");
        if item.move_to(LegacyApprovalCheckpointStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyApprovalCheckpointStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_approval_checkpoint("code-734", "name-892");
        item.status = LegacyApprovalCheckpointStatus::Archived;
        assert!(
            item.move_to(LegacyApprovalCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
