use super::*;

use acme_kit::text;

/// Where a audit event agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventAgreementStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAgreement {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub fingerprint: String,
    pub limit: i64,
    pub channel: String,
    pub key_id: String,
    pub country: String,
    pub unit: String,
    pub status: AuditEventAgreementStatus,
}

impl AuditEventAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventAgreementStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventAgreementStatus) -> bool {
        match self.status {
            AuditEventAgreementStatus::Proposed => {
                return next == AuditEventAgreementStatus::Approved;
            }
            AuditEventAgreementStatus::Approved => {
                match next {
                    AuditEventAgreementStatus::Rejected | AuditEventAgreementStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventAgreementStatus::Rejected => {
                match next {
                    AuditEventAgreementStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AuditEventAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> AuditEventAgreement {
        AuditEventAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            fingerprint: self.fingerprint.clone(),
            limit: self.limit,
            channel: self.channel.clone(),
            key_id: self.key_id.clone(),
            country: self.country.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a audit event agreement in status proposed.
pub fn new_audit_event_agreement(code: &str, title: &str) -> AuditEventAgreement {
    AuditEventAgreement {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        fingerprint: "".to_string(),
        limit: 0,
        channel: "".to_string(),
        key_id: "".to_string(),
        country: "".to_string(),
        unit: "".to_string(),
        status: AuditEventAgreementStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_audit_event_agreement("code-395", "title-292");
        assert_eq!(item.status, AuditEventAgreementStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_audit_event_agreement("code-395", "title-292");
        if item.move_to(AuditEventAgreementStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventAgreementStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_audit_event_agreement("code-395", "title-292");
        item.status = AuditEventAgreementStatus::Withdrawn;
        assert!(
            item.move_to(AuditEventAgreementStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
