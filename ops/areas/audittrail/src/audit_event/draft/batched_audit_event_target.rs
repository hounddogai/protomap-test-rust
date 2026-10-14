use super::*;

use acme_kit::text;

/// Where a batched audit event target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAuditEventTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedAuditEventTargetStatus {
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

impl std::fmt::Display for BatchedAuditEventTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched audit event target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventTarget {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub cipher: String,
    pub maximum: f64,
    pub status: BatchedAuditEventTargetStatus,
}

impl BatchedAuditEventTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAuditEventTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAuditEventTargetStatus) -> bool {
        match self.status {
            BatchedAuditEventTargetStatus::Open => {
                return next == BatchedAuditEventTargetStatus::InReview;
            }
            BatchedAuditEventTargetStatus::InReview => {
                return next == BatchedAuditEventTargetStatus::Resolved;
            }
            BatchedAuditEventTargetStatus::Resolved => {
                match next {
                    BatchedAuditEventTargetStatus::Reopened | BatchedAuditEventTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAuditEventTargetStatus::Reopened => {
                return next == BatchedAuditEventTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedAuditEventTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched audit event target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched audit event target", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.maximum * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAuditEventTarget {
        BatchedAuditEventTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            cipher: self.cipher.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched audit event target in status open.
pub fn new_batched_audit_event_target(code: &str, name: &str) -> BatchedAuditEventTarget {
    BatchedAuditEventTarget {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        cipher: "".to_string(),
        maximum: 0.0,
        status: BatchedAuditEventTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_audit_event_target("code-253", "name-62");
        assert_eq!(item.status, BatchedAuditEventTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_batched_audit_event_target("code-253", "name-62");
        if item.move_to(BatchedAuditEventTargetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAuditEventTargetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_audit_event_target("code-253", "name-62");
        item.status = BatchedAuditEventTargetStatus::Archived;
        assert!(
            item.move_to(BatchedAuditEventTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
