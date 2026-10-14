use super::*;

use acme_kit::text;

/// Where a fallback audit event item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackAuditEventItemStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackAuditEventItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackAuditEventItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback audit event item. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventItem {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub quantity: i64,
    pub title: String,
    pub notes: String,
    pub priority: i64,
    pub summary: String,
    pub description: String,
    pub active: bool,
    pub unit: String,
    pub status: FallbackAuditEventItemStatus,
}

impl FallbackAuditEventItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackAuditEventItemStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackAuditEventItemStatus) -> bool {
        match self.status {
            FallbackAuditEventItemStatus::Requested => {
                return next == FallbackAuditEventItemStatus::Granted;
            }
            FallbackAuditEventItemStatus::Granted => {
                match next {
                    FallbackAuditEventItemStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackAuditEventItemStatus::Revoked => {
                match next {
                    FallbackAuditEventItemStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackAuditEventItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback audit event item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback audit event item", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.priority as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackAuditEventItem {
        FallbackAuditEventItem {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            quantity: self.quantity,
            title: self.title.clone(),
            notes: self.notes.clone(),
            priority: self.priority,
            summary: self.summary.clone(),
            description: self.description.clone(),
            active: self.active,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a fallback audit event item in status requested.
pub fn new_fallback_audit_event_item(code: &str, label: &str) -> FallbackAuditEventItem {
    FallbackAuditEventItem {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        quantity: 0,
        title: "".to_string(),
        notes: "".to_string(),
        priority: 0,
        summary: "".to_string(),
        description: "".to_string(),
        active: false,
        unit: "".to_string(),
        status: FallbackAuditEventItemStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_audit_event_item("code-994", "label-226");
        assert_eq!(item.status, FallbackAuditEventItemStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_fallback_audit_event_item("code-994", "label-226");
        if item.move_to(FallbackAuditEventItemStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackAuditEventItemStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_audit_event_item("code-994", "label-226");
        item.status = FallbackAuditEventItemStatus::Expired;
        assert!(
            item.move_to(FallbackAuditEventItemStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
