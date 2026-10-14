use super::*;

use acme_kit::text;

/// Where a budget burn score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BudgetBurnScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BudgetBurnScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScore {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub minimum: f64,
    pub reviewer: String,
    pub status: BudgetBurnScoreStatus,
}

impl BudgetBurnScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnScoreStatus) -> bool {
        match self.status {
            BudgetBurnScoreStatus::Requested => {
                match next {
                    BudgetBurnScoreStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnScoreStatus::Granted => {
                match next {
                    BudgetBurnScoreStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnScoreStatus::Revoked => {
                match next {
                    BudgetBurnScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: BudgetBurnScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> BudgetBurnScore {
        BudgetBurnScore {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            minimum: self.minimum,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a budget burn score in status requested.
pub fn new_budget_burn_score(code: &str, name: &str) -> BudgetBurnScore {
    BudgetBurnScore {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        minimum: 0.0,
        reviewer: "".to_string(),
        status: BudgetBurnScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_budget_burn_score("code-890", "name-150");
        assert_eq!(item.status, BudgetBurnScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_budget_burn_score("code-890", "name-150");
        if item.move_to(BudgetBurnScoreStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnScoreStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_budget_burn_score("code-890", "name-150");
        item.status = BudgetBurnScoreStatus::Expired;
        assert!(
            item.move_to(BudgetBurnScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
