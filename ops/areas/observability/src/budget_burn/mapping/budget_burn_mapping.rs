use super::*;

use acme_kit::text;

/// Where a budget burn mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BudgetBurnMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BudgetBurnMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn mapping as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMapping {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub capacity: i64,
    pub limit: i64,
    pub quota_units: i64,
    pub status: BudgetBurnMappingStatus,
}

impl BudgetBurnMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnMappingStatus) -> bool {
        match self.status {
            BudgetBurnMappingStatus::Requested => {
                return next == BudgetBurnMappingStatus::Expired;
            }
            BudgetBurnMappingStatus::Granted => {
                match next {
                    BudgetBurnMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnMappingStatus::Revoked => {
                match next {
                    BudgetBurnMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: BudgetBurnMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quota_units as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnMapping {
        BudgetBurnMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            capacity: self.capacity,
            limit: self.limit,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a budget burn mapping in status requested.
pub fn new_budget_burn_mapping(code: &str, label: &str) -> BudgetBurnMapping {
    BudgetBurnMapping {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        capacity: 0,
        limit: 0,
        quota_units: 0,
        status: BudgetBurnMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_budget_burn_mapping("code-844", "label-601");
        assert_eq!(item.status, BudgetBurnMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_budget_burn_mapping("code-844", "label-601");
        if item.move_to(BudgetBurnMappingStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnMappingStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_budget_burn_mapping("code-844", "label-601");
        item.status = BudgetBurnMappingStatus::Expired;
        assert!(
            item.move_to(BudgetBurnMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
