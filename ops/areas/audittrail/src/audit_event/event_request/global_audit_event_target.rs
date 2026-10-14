use super::*;

use acme_kit::text;

/// Where a global audit event target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAuditEventTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalAuditEventTargetStatus {
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

impl std::fmt::Display for GlobalAuditEventTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global audit event target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventTarget {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub owner_id: String,
    pub approved: bool,
    pub port: i64,
    pub priority: i64,
    pub region: String,
    pub sequence: i64,
    pub source: String,
    pub reviewer: String,
    pub status: GlobalAuditEventTargetStatus,
}

impl GlobalAuditEventTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAuditEventTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAuditEventTargetStatus) -> bool {
        match self.status {
            GlobalAuditEventTargetStatus::Open => {
                match next {
                    GlobalAuditEventTargetStatus::InReview | GlobalAuditEventTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventTargetStatus::InReview => {
                return next == GlobalAuditEventTargetStatus::Reopened
                    || next == GlobalAuditEventTargetStatus::Archived;
            }
            GlobalAuditEventTargetStatus::Resolved => {
                match next {
                    GlobalAuditEventTargetStatus::Reopened | GlobalAuditEventTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventTargetStatus::Reopened => {
                return next == GlobalAuditEventTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalAuditEventTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global audit event target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.revision as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAuditEventTarget {
        GlobalAuditEventTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            owner_id: self.owner_id.clone(),
            approved: self.approved,
            port: self.port,
            priority: self.priority,
            region: self.region.clone(),
            sequence: self.sequence,
            source: self.source.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a global audit event target in status open.
pub fn new_global_audit_event_target(code: &str, label: &str) -> GlobalAuditEventTarget {
    GlobalAuditEventTarget {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        owner_id: "".to_string(),
        approved: false,
        port: 0,
        priority: 0,
        region: "".to_string(),
        sequence: 0,
        source: "".to_string(),
        reviewer: "".to_string(),
        status: GlobalAuditEventTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_global_audit_event_target("code-191", "label-554");
        assert_eq!(item.status, GlobalAuditEventTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_global_audit_event_target("code-191", "label-554");
        if item.move_to(GlobalAuditEventTargetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalAuditEventTargetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_global_audit_event_target("code-191", "label-554");
        item.status = GlobalAuditEventTargetStatus::Archived;
        assert!(
            item.move_to(GlobalAuditEventTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
