use super::*;

use acme_kit::text;

/// Where a audit event notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AuditEventNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AuditEventNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventNotice {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub summary: String,
    pub floor_value: f64,
    pub ratio: f64,
    pub maximum: f64,
    pub contact_email: String,
    pub currency: String,
    pub status: AuditEventNoticeStatus,
}

impl AuditEventNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventNoticeStatus) -> bool {
        match self.status {
            AuditEventNoticeStatus::Draft => {
                return next == AuditEventNoticeStatus::Active;
            }
            AuditEventNoticeStatus::Pending => {
                return next == AuditEventNoticeStatus::Suspended || next == AuditEventNoticeStatus::Closed;
            }
            AuditEventNoticeStatus::Active => {
                return next == AuditEventNoticeStatus::Suspended || next == AuditEventNoticeStatus::Closed;
            }
            AuditEventNoticeStatus::Suspended => {
                return next == AuditEventNoticeStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.floor_value * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventNotice {
        AuditEventNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            summary: self.summary.clone(),
            floor_value: self.floor_value,
            ratio: self.ratio,
            maximum: self.maximum,
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a audit event notice in status draft.
pub fn new_audit_event_notice(code: &str, title: &str) -> AuditEventNotice {
    AuditEventNotice {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        summary: "".to_string(),
        floor_value: 0.0,
        ratio: 0.0,
        maximum: 0.0,
        contact_email: "".to_string(),
        currency: "".to_string(),
        status: AuditEventNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_audit_event_notice("code-427", "title-812");
        assert_eq!(item.status, AuditEventNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_audit_event_notice("code-427", "title-812");
        if item.move_to(AuditEventNoticeStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventNoticeStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_audit_event_notice("code-427", "title-812");
        item.status = AuditEventNoticeStatus::Closed;
        assert!(
            item.move_to(AuditEventNoticeStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
