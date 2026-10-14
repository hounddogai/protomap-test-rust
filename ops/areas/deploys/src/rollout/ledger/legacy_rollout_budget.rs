use super::*;

use acme_kit::text;

/// Where a legacy rollout budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRolloutBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyRolloutBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyRolloutBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollout budget as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutBudget {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub limit: i64,
    pub contact_email: String,
    pub label: String,
    pub cost: f64,
    pub status: LegacyRolloutBudgetStatus,
}

impl LegacyRolloutBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRolloutBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRolloutBudgetStatus) -> bool {
        match self.status {
            LegacyRolloutBudgetStatus::Proposed => {
                match next {
                    LegacyRolloutBudgetStatus::Approved | LegacyRolloutBudgetStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutBudgetStatus::Approved => {
                return next == LegacyRolloutBudgetStatus::Rejected;
            }
            LegacyRolloutBudgetStatus::Rejected => {
                match next {
                    LegacyRolloutBudgetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyRolloutBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollout budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.cost + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRolloutBudget {
        LegacyRolloutBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            limit: self.limit,
            contact_email: self.contact_email.clone(),
            label: self.label.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy rollout budget in status proposed.
pub fn new_legacy_rollout_budget(code: &str, name: &str) -> LegacyRolloutBudget {
    LegacyRolloutBudget {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        limit: 0,
        contact_email: "".to_string(),
        label: "".to_string(),
        cost: 0.0,
        status: LegacyRolloutBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_rollout_budget("code-45", "name-419");
        assert_eq!(item.status, LegacyRolloutBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_legacy_rollout_budget("code-45", "name-419");
        if item.move_to(LegacyRolloutBudgetStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRolloutBudgetStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_rollout_budget("code-45", "name-419");
        item.status = LegacyRolloutBudgetStatus::Withdrawn;
        assert!(
            item.move_to(LegacyRolloutBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
