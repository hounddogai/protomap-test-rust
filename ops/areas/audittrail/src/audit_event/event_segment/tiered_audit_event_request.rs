use super::*;

use acme_kit::text;

/// Where a tiered audit event request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAuditEventRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredAuditEventRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TieredAuditEventRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered audit event request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequest {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub notes: String,
    pub budget: f64,
    pub namespace: String,
    pub currency: String,
    pub status: TieredAuditEventRequestStatus,
}

impl TieredAuditEventRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAuditEventRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAuditEventRequestStatus) -> bool {
        match self.status {
            TieredAuditEventRequestStatus::New => {
                match next {
                    TieredAuditEventRequestStatus::Queued | TieredAuditEventRequestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAuditEventRequestStatus::Queued => {
                return next == TieredAuditEventRequestStatus::Done;
            }
            TieredAuditEventRequestStatus::Running => {
                match next {
                    TieredAuditEventRequestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAuditEventRequestStatus::Done => {
                match next {
                    TieredAuditEventRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredAuditEventRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered audit event request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> TieredAuditEventRequest {
        TieredAuditEventRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            notes: self.notes.clone(),
            budget: self.budget,
            namespace: self.namespace.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a tiered audit event request in status new.
pub fn new_tiered_audit_event_request(code: &str, label: &str) -> TieredAuditEventRequest {
    TieredAuditEventRequest {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        notes: "".to_string(),
        budget: 0.0,
        namespace: "".to_string(),
        currency: "".to_string(),
        status: TieredAuditEventRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_audit_event_request("code-898", "label-113");
        assert_eq!(item.status, TieredAuditEventRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_audit_event_request("code-898", "label-113");
        if item.move_to(TieredAuditEventRequestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredAuditEventRequestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_audit_event_request("code-898", "label-113");
        item.status = TieredAuditEventRequestStatus::Failed;
        assert!(
            item.move_to(TieredAuditEventRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
