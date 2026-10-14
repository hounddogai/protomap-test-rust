use super::*;

use acme_kit::text;

/// Where a evidence audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EvidenceAuditStatus {
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

impl std::fmt::Display for EvidenceAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAudit {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub capacity: i64,
    pub tenant_id: String,
    pub expires_at: i64,
    pub issuer: String,
    pub title: String,
    pub contact_email: String,
    pub status: EvidenceAuditStatus,
}

impl EvidenceAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceAuditStatus) -> bool {
        match self.status {
            EvidenceAuditStatus::Open => {
                match next {
                    EvidenceAuditStatus::Resolved | EvidenceAuditStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceAuditStatus::InReview => {
                return next == EvidenceAuditStatus::Resolved || next == EvidenceAuditStatus::Archived;
            }
            EvidenceAuditStatus::Resolved => {
                return next == EvidenceAuditStatus::Reopened || next == EvidenceAuditStatus::Archived;
            }
            EvidenceAuditStatus::Reopened => {
                match next {
                    EvidenceAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: EvidenceAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.capacity as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceAudit {
        EvidenceAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            capacity: self.capacity,
            tenant_id: self.tenant_id.clone(),
            expires_at: self.expires_at,
            issuer: self.issuer.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a evidence audit in status open.
pub fn new_evidence_audit(code: &str, name: &str) -> EvidenceAudit {
    EvidenceAudit {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        capacity: 0,
        tenant_id: "".to_string(),
        expires_at: 0,
        issuer: "".to_string(),
        title: "".to_string(),
        contact_email: "".to_string(),
        status: EvidenceAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_evidence_audit("code-881", "name-719");
        assert_eq!(item.status, EvidenceAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_evidence_audit("code-881", "name-719");
        if item.move_to(EvidenceAuditStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceAuditStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_evidence_audit("code-881", "name-719");
        item.status = EvidenceAuditStatus::Archived;
        assert!(
            item.move_to(EvidenceAuditStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
