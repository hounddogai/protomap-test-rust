use super::*;

use acme_kit::text;

/// Where a audit event request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AuditEventRequestStatus {
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

impl std::fmt::Display for AuditEventRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRequest {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub visible: bool,
    pub cipher: String,
    pub status: AuditEventRequestStatus,
}

impl AuditEventRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventRequestStatus) -> bool {
        match self.status {
            AuditEventRequestStatus::New => {
                return next == AuditEventRequestStatus::Queued;
            }
            AuditEventRequestStatus::Queued => {
                return next == AuditEventRequestStatus::Failed;
            }
            AuditEventRequestStatus::Running => {
                match next {
                    AuditEventRequestStatus::Done | AuditEventRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventRequestStatus::Done => {
                match next {
                    AuditEventRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: AuditEventRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event request {} cannot move from {} to {}",
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
        self.discount
    }

    pub fn copy(&self) -> AuditEventRequest {
        AuditEventRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            visible: self.visible,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a audit event request in status new.
pub fn new_audit_event_request(code: &str, label: &str) -> AuditEventRequest {
    AuditEventRequest {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        visible: false,
        cipher: "".to_string(),
        status: AuditEventRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_audit_event_request("code-671", "label-792");
        assert_eq!(item.status, AuditEventRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_audit_event_request("code-671", "label-792");
        if item.move_to(AuditEventRequestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventRequestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_audit_event_request("code-671", "label-792");
        item.status = AuditEventRequestStatus::Failed;
        assert!(
            item.move_to(AuditEventRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
