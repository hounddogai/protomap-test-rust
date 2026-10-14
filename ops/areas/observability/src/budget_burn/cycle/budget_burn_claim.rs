use super::*;

use acme_kit::text;

/// Where a budget burn claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BudgetBurnClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BudgetBurnClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnClaim {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub reviewer: String,
    pub timezone: String,
    pub amount: f64,
    pub status: BudgetBurnClaimStatus,
}

impl BudgetBurnClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnClaimStatus) -> bool {
        match self.status {
            BudgetBurnClaimStatus::Created => {
                match next {
                    BudgetBurnClaimStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnClaimStatus::Validated => {
                return next == BudgetBurnClaimStatus::Retired;
            }
            BudgetBurnClaimStatus::Published => {
                match next {
                    BudgetBurnClaimStatus::Retired => {
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

    pub fn move_to(&mut self, next: BudgetBurnClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn claim", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> BudgetBurnClaim {
        BudgetBurnClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            reviewer: self.reviewer.clone(),
            timezone: self.timezone.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a budget burn claim in status created.
pub fn new_budget_burn_claim(code: &str, name: &str) -> BudgetBurnClaim {
    BudgetBurnClaim {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        reviewer: "".to_string(),
        timezone: "".to_string(),
        amount: 0.0,
        status: BudgetBurnClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_budget_burn_claim("code-736", "name-367");
        assert_eq!(item.status, BudgetBurnClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_budget_burn_claim("code-736", "name-367");
        if item.move_to(BudgetBurnClaimStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnClaimStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_budget_burn_claim("code-736", "name-367");
        item.status = BudgetBurnClaimStatus::Retired;
        assert!(
            item.move_to(BudgetBurnClaimStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
