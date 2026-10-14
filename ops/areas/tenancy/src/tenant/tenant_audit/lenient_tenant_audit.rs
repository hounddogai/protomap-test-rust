use super::*;

use acme_kit::text;

/// Where a lenient tenant audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTenantAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientTenantAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientTenantAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient tenant audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantAudit {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub amount: f64,
    pub score: f64,
    pub quota_units: i64,
    pub status: LenientTenantAuditStatus,
}

impl LenientTenantAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTenantAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTenantAuditStatus) -> bool {
        match self.status {
            LenientTenantAuditStatus::Planned => {
                return next == LenientTenantAuditStatus::Completed;
            }
            LenientTenantAuditStatus::Scheduled => {
                return next == LenientTenantAuditStatus::InProgress;
            }
            LenientTenantAuditStatus::InProgress => {
                return next == LenientTenantAuditStatus::Completed || next == LenientTenantAuditStatus::Cancelled;
            }
            LenientTenantAuditStatus::Completed => {
                return next == LenientTenantAuditStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientTenantAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient tenant audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        (self.amount * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTenantAudit {
        LenientTenantAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            amount: self.amount,
            score: self.score,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient tenant audit in status planned.
pub fn new_lenient_tenant_audit(code: &str, title: &str) -> LenientTenantAudit {
    LenientTenantAudit {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        amount: 0.0,
        score: 0.0,
        quota_units: 0,
        status: LenientTenantAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_tenant_audit("code-365", "title-913");
        assert_eq!(item.status, LenientTenantAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_lenient_tenant_audit("code-365", "title-913");
        if item.move_to(LenientTenantAuditStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTenantAuditStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_tenant_audit("code-365", "title-913");
        item.status = LenientTenantAuditStatus::Cancelled;
        assert!(
            item.move_to(LenientTenantAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
