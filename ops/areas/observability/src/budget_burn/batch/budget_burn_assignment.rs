use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a budget burn assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BudgetBurnAssignmentStatus {
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

impl std::fmt::Display for BudgetBurnAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAssignment {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub rotation_days: i64,
    pub phone: String,
    pub created_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub duration_minutes: i64,
    pub grace_days: i64,
    pub reviewer: String,
    pub checksum: String,
    pub status: BudgetBurnAssignmentStatus,
}

impl BudgetBurnAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnAssignmentStatus) -> bool {
        match self.status {
            BudgetBurnAssignmentStatus::Open => {
                match next {
                    BudgetBurnAssignmentStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnAssignmentStatus::InReview => {
                return next == BudgetBurnAssignmentStatus::Reopened;
            }
            BudgetBurnAssignmentStatus::Resolved => {
                return next == BudgetBurnAssignmentStatus::Reopened;
            }
            BudgetBurnAssignmentStatus::Reopened => {
                return next == BudgetBurnAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn effective_volume(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnAssignment {
        BudgetBurnAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            rotation_days: self.rotation_days,
            phone: self.phone.clone(),
            created_at: self.created_at,
            attributes: self.attributes.clone(),
            duration_minutes: self.duration_minutes,
            grace_days: self.grace_days,
            reviewer: self.reviewer.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a budget burn assignment in status open.
pub fn new_budget_burn_assignment(code: &str, name: &str) -> BudgetBurnAssignment {
    BudgetBurnAssignment {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        rotation_days: 0,
        phone: "".to_string(),
        created_at: 0,
        attributes: BTreeMap::new(),
        duration_minutes: 0,
        grace_days: 0,
        reviewer: "".to_string(),
        checksum: "".to_string(),
        status: BudgetBurnAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_budget_burn_assignment("code-348", "name-800");
        assert_eq!(item.status, BudgetBurnAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_budget_burn_assignment("code-348", "name-800");
        if item.move_to(BudgetBurnAssignmentStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnAssignmentStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_budget_burn_assignment("code-348", "name-800");
        item.status = BudgetBurnAssignmentStatus::Archived;
        assert!(
            item.move_to(BudgetBurnAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
