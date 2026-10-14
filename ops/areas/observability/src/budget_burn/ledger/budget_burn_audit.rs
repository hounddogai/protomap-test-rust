use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a budget burn audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BudgetBurnAuditStatus {
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

impl std::fmt::Display for BudgetBurnAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAudit {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub fingerprint: String,
    pub priority: i64,
    pub title: String,
    pub status: BudgetBurnAuditStatus,
}

impl BudgetBurnAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnAuditStatus) -> bool {
        match self.status {
            BudgetBurnAuditStatus::Open => {
                return next == BudgetBurnAuditStatus::Reopened;
            }
            BudgetBurnAuditStatus::InReview => {
                match next {
                    BudgetBurnAuditStatus::Reopened | BudgetBurnAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnAuditStatus::Resolved => {
                return next == BudgetBurnAuditStatus::Reopened || next == BudgetBurnAuditStatus::Archived;
            }
            BudgetBurnAuditStatus::Reopened => {
                match next {
                    BudgetBurnAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: BudgetBurnAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.priority as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnAudit {
        BudgetBurnAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            fingerprint: self.fingerprint.clone(),
            priority: self.priority,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a budget burn audit in status open.
pub fn new_budget_burn_audit(code: &str, label: &str) -> BudgetBurnAudit {
    BudgetBurnAudit {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        fingerprint: "".to_string(),
        priority: 0,
        title: "".to_string(),
        status: BudgetBurnAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_budget_burn_audit("code-710", "label-848");
        assert_eq!(item.status, BudgetBurnAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_budget_burn_audit("code-710", "label-848");
        if item.move_to(BudgetBurnAuditStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnAuditStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_budget_burn_audit("code-710", "label-848");
        item.status = BudgetBurnAuditStatus::Archived;
        assert!(
            item.move_to(BudgetBurnAuditStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
