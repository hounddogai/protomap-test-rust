use super::*;

use acme_kit::text;

/// Where a capped budget burn audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBudgetBurnAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedBudgetBurnAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedBudgetBurnAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped budget burn audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAudit {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub locale: String,
    pub window_days: i64,
    pub ttl_seconds: i64,
    pub category: String,
    pub status: CappedBudgetBurnAuditStatus,
}

impl CappedBudgetBurnAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBudgetBurnAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBudgetBurnAuditStatus) -> bool {
        match self.status {
            CappedBudgetBurnAuditStatus::Requested => {
                return next == CappedBudgetBurnAuditStatus::Granted || next == CappedBudgetBurnAuditStatus::Expired;
            }
            CappedBudgetBurnAuditStatus::Granted => {
                return next == CappedBudgetBurnAuditStatus::Revoked;
            }
            CappedBudgetBurnAuditStatus::Revoked => {
                match next {
                    CappedBudgetBurnAuditStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedBudgetBurnAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped budget burn audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedBudgetBurnAudit {
        CappedBudgetBurnAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            window_days: self.window_days,
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped budget burn audit in status requested.
pub fn new_capped_budget_burn_audit(code: &str, label: &str) -> CappedBudgetBurnAudit {
    CappedBudgetBurnAudit {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        grace_days: 0,
        locale: "".to_string(),
        window_days: 0,
        ttl_seconds: 0,
        category: "".to_string(),
        status: CappedBudgetBurnAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_budget_burn_audit("code-127", "label-74");
        assert_eq!(item.status, CappedBudgetBurnAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_budget_burn_audit("code-127", "label-74");
        if item.move_to(CappedBudgetBurnAuditStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedBudgetBurnAuditStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_budget_burn_audit("code-127", "label-74");
        item.status = CappedBudgetBurnAuditStatus::Expired;
        assert!(
            item.move_to(CappedBudgetBurnAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
