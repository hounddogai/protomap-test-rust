use super::*;

use acme_kit::text;

/// Where a tiered ttl policy snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTtlPolicySnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredTtlPolicySnapshotStatus {
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

impl std::fmt::Display for TieredTtlPolicySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered ttl policy snapshot as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicySnapshot {
    pub code: String,
    pub title: String,
    pub description: String,
    pub batch_size: i64,
    pub currency: String,
    pub score: f64,
    pub cipher: String,
    pub status: TieredTtlPolicySnapshotStatus,
}

impl TieredTtlPolicySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTtlPolicySnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTtlPolicySnapshotStatus) -> bool {
        match self.status {
            TieredTtlPolicySnapshotStatus::Open => {
                return next == TieredTtlPolicySnapshotStatus::Reopened
                    || next == TieredTtlPolicySnapshotStatus::Archived;
            }
            TieredTtlPolicySnapshotStatus::InReview => {
                match next {
                    TieredTtlPolicySnapshotStatus::Reopened | TieredTtlPolicySnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTtlPolicySnapshotStatus::Resolved => {
                match next {
                    TieredTtlPolicySnapshotStatus::Reopened | TieredTtlPolicySnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTtlPolicySnapshotStatus::Reopened => {
                match next {
                    TieredTtlPolicySnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredTtlPolicySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered ttl policy snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered ttl policy snapshot", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.batch_size as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTtlPolicySnapshot {
        TieredTtlPolicySnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            batch_size: self.batch_size,
            currency: self.currency.clone(),
            score: self.score,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered ttl policy snapshot in status open.
pub fn new_tiered_ttl_policy_snapshot(code: &str, title: &str) -> TieredTtlPolicySnapshot {
    TieredTtlPolicySnapshot {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        batch_size: 0,
        currency: "".to_string(),
        score: 0.0,
        cipher: "".to_string(),
        status: TieredTtlPolicySnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_ttl_policy_snapshot("code-332", "title-87");
        assert_eq!(item.status, TieredTtlPolicySnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tiered_ttl_policy_snapshot("code-332", "title-87");
        if item.move_to(TieredTtlPolicySnapshotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTtlPolicySnapshotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_ttl_policy_snapshot("code-332", "title-87");
        item.status = TieredTtlPolicySnapshotStatus::Archived;
        assert!(
            item.move_to(TieredTtlPolicySnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
