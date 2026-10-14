use super::*;

use acme_kit::text;

/// Where a audit event adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AuditEventAdjustmentStatus {
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

impl std::fmt::Display for AuditEventAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAdjustment {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub currency: String,
    pub source: String,
    pub issuer: String,
    pub minimum: f64,
    pub pinned: bool,
    pub updated_at: i64,
    pub window_days: i64,
    pub quantity: i64,
    pub status: AuditEventAdjustmentStatus,
}

impl AuditEventAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventAdjustmentStatus) -> bool {
        match self.status {
            AuditEventAdjustmentStatus::Open => {
                return next == AuditEventAdjustmentStatus::Resolved;
            }
            AuditEventAdjustmentStatus::InReview => {
                return next == AuditEventAdjustmentStatus::Resolved || next == AuditEventAdjustmentStatus::Archived;
            }
            AuditEventAdjustmentStatus::Resolved => {
                return next == AuditEventAdjustmentStatus::Archived;
            }
            AuditEventAdjustmentStatus::Reopened => {
                match next {
                    AuditEventAdjustmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: AuditEventAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventAdjustment {
        AuditEventAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            source: self.source.clone(),
            issuer: self.issuer.clone(),
            minimum: self.minimum,
            pinned: self.pinned,
            updated_at: self.updated_at,
            window_days: self.window_days,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a audit event adjustment in status open.
pub fn new_audit_event_adjustment(code: &str, name: &str) -> AuditEventAdjustment {
    AuditEventAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        currency: "".to_string(),
        source: "".to_string(),
        issuer: "".to_string(),
        minimum: 0.0,
        pinned: false,
        updated_at: 0,
        window_days: 0,
        quantity: 0,
        status: AuditEventAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_audit_event_adjustment("code-160", "name-119");
        assert_eq!(item.status, AuditEventAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_audit_event_adjustment("code-160", "name-119");
        if item.move_to(AuditEventAdjustmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventAdjustmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_audit_event_adjustment("code-160", "name-119");
        item.status = AuditEventAdjustmentStatus::Archived;
        assert!(
            item.move_to(AuditEventAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
