use super::*;

use acme_kit::text;

/// Where a rollback score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollbackScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollbackScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScore {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub capacity: i64,
    pub ceiling: f64,
    pub tenant_id: String,
    pub sequence: i64,
    pub maximum: f64,
    pub priority: i64,
    pub checksum: String,
    pub status: RollbackScoreStatus,
}

impl RollbackScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackScoreStatus) -> bool {
        match self.status {
            RollbackScoreStatus::Requested => {
                return next == RollbackScoreStatus::Revoked || next == RollbackScoreStatus::Expired;
            }
            RollbackScoreStatus::Granted => {
                return next == RollbackScoreStatus::Revoked;
            }
            RollbackScoreStatus::Revoked => {
                match next {
                    RollbackScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollbackScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback score", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.capacity as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackScore {
        RollbackScore {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            capacity: self.capacity,
            ceiling: self.ceiling,
            tenant_id: self.tenant_id.clone(),
            sequence: self.sequence,
            maximum: self.maximum,
            priority: self.priority,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback score in status requested.
pub fn new_rollback_score(code: &str, label: &str) -> RollbackScore {
    RollbackScore {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        capacity: 0,
        ceiling: 0.0,
        tenant_id: "".to_string(),
        sequence: 0,
        maximum: 0.0,
        priority: 0,
        checksum: "".to_string(),
        status: RollbackScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollback_score("code-309", "label-622");
        assert_eq!(item.status, RollbackScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rollback_score("code-309", "label-622");
        if item.move_to(RollbackScoreStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackScoreStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollback_score("code-309", "label-622");
        item.status = RollbackScoreStatus::Expired;
        assert!(
            item.move_to(RollbackScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
