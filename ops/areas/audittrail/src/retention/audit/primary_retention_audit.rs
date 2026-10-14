use super::*;

use acme_kit::text;

/// Where a primary retention audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRetentionAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryRetentionAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryRetentionAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary retention audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionAudit {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub discount: f64,
    pub summary: String,
    pub unit: String,
    pub status: PrimaryRetentionAuditStatus,
}

impl PrimaryRetentionAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRetentionAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRetentionAuditStatus) -> bool {
        match self.status {
            PrimaryRetentionAuditStatus::Requested => {
                return next == PrimaryRetentionAuditStatus::Revoked || next == PrimaryRetentionAuditStatus::Expired;
            }
            PrimaryRetentionAuditStatus::Granted => {
                return next == PrimaryRetentionAuditStatus::Revoked;
            }
            PrimaryRetentionAuditStatus::Revoked => {
                return next == PrimaryRetentionAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryRetentionAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary retention audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.discount + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRetentionAudit {
        PrimaryRetentionAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            discount: self.discount,
            summary: self.summary.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary retention audit in status requested.
pub fn new_primary_retention_audit(code: &str, name: &str) -> PrimaryRetentionAudit {
    PrimaryRetentionAudit {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        discount: 0.0,
        summary: "".to_string(),
        unit: "".to_string(),
        status: PrimaryRetentionAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_retention_audit("code-364", "name-568");
        assert_eq!(item.status, PrimaryRetentionAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_retention_audit("code-364", "name-568");
        if item.move_to(PrimaryRetentionAuditStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRetentionAuditStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_retention_audit("code-364", "name-568");
        item.status = PrimaryRetentionAuditStatus::Expired;
        assert!(
            item.move_to(PrimaryRetentionAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
