use super::*;

use acme_kit::text;

/// Where a verifier audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAudit {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub name: String,
    pub duration_minutes: i64,
    pub priority: i64,
    pub weight: f64,
    pub reviewer: String,
    pub approved: bool,
    pub status: VerifierAuditStatus,
}

impl VerifierAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierAuditStatus) -> bool {
        match self.status {
            VerifierAuditStatus::Requested => {
                return next == VerifierAuditStatus::Granted || next == VerifierAuditStatus::Revoked;
            }
            VerifierAuditStatus::Granted => {
                match next {
                    VerifierAuditStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierAuditStatus::Revoked => {
                return next == VerifierAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierAudit {
        VerifierAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            priority: self.priority,
            weight: self.weight,
            reviewer: self.reviewer.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a verifier audit in status requested.
pub fn new_verifier_audit(code: &str, label: &str) -> VerifierAudit {
    VerifierAudit {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        name: "".to_string(),
        duration_minutes: 0,
        priority: 0,
        weight: 0.0,
        reviewer: "".to_string(),
        approved: false,
        status: VerifierAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_audit("code-309", "label-385");
        assert_eq!(item.status, VerifierAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_verifier_audit("code-309", "label-385");
        if item.move_to(VerifierAuditStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierAuditStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_audit("code-309", "label-385");
        item.status = VerifierAuditStatus::Expired;
        assert!(
            item.move_to(VerifierAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
