use super::*;

use acme_kit::text;

/// Where a pending redaction slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRedactionSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingRedactionSlotStatus {
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

impl std::fmt::Display for PendingRedactionSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending redaction slot as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionSlot {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub timezone: String,
    pub revision: i64,
    pub scope: String,
    pub issuer: String,
    pub status: PendingRedactionSlotStatus,
}

impl PendingRedactionSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRedactionSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRedactionSlotStatus) -> bool {
        match self.status {
            PendingRedactionSlotStatus::Open => {
                match next {
                    PendingRedactionSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRedactionSlotStatus::InReview => {
                match next {
                    PendingRedactionSlotStatus::Reopened | PendingRedactionSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRedactionSlotStatus::Resolved => {
                return next == PendingRedactionSlotStatus::Archived;
            }
            PendingRedactionSlotStatus::Reopened => {
                return next == PendingRedactionSlotStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRedactionSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending redaction slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.burst as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRedactionSlot {
        PendingRedactionSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            timezone: self.timezone.clone(),
            revision: self.revision,
            scope: self.scope.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending redaction slot in status open.
pub fn new_pending_redaction_slot(code: &str, label: &str) -> PendingRedactionSlot {
    PendingRedactionSlot {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        timezone: "".to_string(),
        revision: 0,
        scope: "".to_string(),
        issuer: "".to_string(),
        status: PendingRedactionSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_redaction_slot("code-95", "label-706");
        assert_eq!(item.status, PendingRedactionSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_pending_redaction_slot("code-95", "label-706");
        if item.move_to(PendingRedactionSlotStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRedactionSlotStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_redaction_slot("code-95", "label-706");
        item.status = PendingRedactionSlotStatus::Archived;
        assert!(
            item.move_to(PendingRedactionSlotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
