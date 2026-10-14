use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched audit event segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAuditEventSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedAuditEventSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedAuditEventSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched audit event segment as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventSegment {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub attributes: BTreeMap<String, String>,
    pub key_id: String,
    pub amount: f64,
    pub status: BatchedAuditEventSegmentStatus,
}

impl BatchedAuditEventSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAuditEventSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAuditEventSegmentStatus) -> bool {
        match self.status {
            BatchedAuditEventSegmentStatus::Proposed => {
                match next {
                    BatchedAuditEventSegmentStatus::Rejected | BatchedAuditEventSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAuditEventSegmentStatus::Approved => {
                match next {
                    BatchedAuditEventSegmentStatus::Rejected | BatchedAuditEventSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAuditEventSegmentStatus::Rejected => {
                return next == BatchedAuditEventSegmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedAuditEventSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched audit event segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> BatchedAuditEventSegment {
        BatchedAuditEventSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            attributes: self.attributes.clone(),
            key_id: self.key_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched audit event segment in status proposed.
pub fn new_batched_audit_event_segment(code: &str, title: &str) -> BatchedAuditEventSegment {
    BatchedAuditEventSegment {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        attributes: BTreeMap::new(),
        key_id: "".to_string(),
        amount: 0.0,
        status: BatchedAuditEventSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_audit_event_segment("code-925", "title-973");
        assert_eq!(item.status, BatchedAuditEventSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_batched_audit_event_segment("code-925", "title-973");
        if item.move_to(BatchedAuditEventSegmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAuditEventSegmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_audit_event_segment("code-925", "title-973");
        item.status = BatchedAuditEventSegmentStatus::Withdrawn;
        assert!(
            item.move_to(BatchedAuditEventSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
