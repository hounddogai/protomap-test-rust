use super::*;

use acme_kit::text;

/// Where a capped audit event request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedAuditEventRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedAuditEventRequestStatus {
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

impl std::fmt::Display for CappedAuditEventRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped audit event request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventRequest {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub quota_units: i64,
    pub budget: f64,
    pub currency: String,
    pub discount: f64,
    pub quantity: i64,
    pub principal: String,
    pub status: CappedAuditEventRequestStatus,
}

impl CappedAuditEventRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedAuditEventRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedAuditEventRequestStatus) -> bool {
        match self.status {
            CappedAuditEventRequestStatus::New => {
                return next == CappedAuditEventRequestStatus::Failed;
            }
            CappedAuditEventRequestStatus::Queued => {
                return next == CappedAuditEventRequestStatus::Running || next == CappedAuditEventRequestStatus::Failed;
            }
            CappedAuditEventRequestStatus::Running => {
                return next == CappedAuditEventRequestStatus::Done || next == CappedAuditEventRequestStatus::Failed;
            }
            CappedAuditEventRequestStatus::Done => {
                return next == CappedAuditEventRequestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedAuditEventRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped audit event request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped audit event request", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quantity as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedAuditEventRequest {
        CappedAuditEventRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            owner_id: self.owner_id.clone(),
            quota_units: self.quota_units,
            budget: self.budget,
            currency: self.currency.clone(),
            discount: self.discount,
            quantity: self.quantity,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped audit event request in status new.
pub fn new_capped_audit_event_request(code: &str, name: &str) -> CappedAuditEventRequest {
    CappedAuditEventRequest {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        owner_id: "".to_string(),
        quota_units: 0,
        budget: 0.0,
        currency: "".to_string(),
        discount: 0.0,
        quantity: 0,
        principal: "".to_string(),
        status: CappedAuditEventRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_audit_event_request("code-693", "name-576");
        assert_eq!(item.status, CappedAuditEventRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_capped_audit_event_request("code-693", "name-576");
        if item.move_to(CappedAuditEventRequestStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedAuditEventRequestStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_audit_event_request("code-693", "name-576");
        item.status = CappedAuditEventRequestStatus::Failed;
        assert!(
            item.move_to(CappedAuditEventRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
