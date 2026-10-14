use super::*;

use acme_kit::text;

/// Where a budget burn rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BudgetBurnRuleStatus {
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

impl std::fmt::Display for BudgetBurnRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRule {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub issuer: String,
    pub rotation_days: i64,
    pub pinned: bool,
    pub country: String,
    pub timezone: String,
    pub region: String,
    pub rate: f64,
    pub status: BudgetBurnRuleStatus,
}

impl BudgetBurnRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnRuleStatus) -> bool {
        match self.status {
            BudgetBurnRuleStatus::Planned => {
                return next == BudgetBurnRuleStatus::Scheduled;
            }
            BudgetBurnRuleStatus::Scheduled => {
                match next {
                    BudgetBurnRuleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnRuleStatus::InProgress => {
                match next {
                    BudgetBurnRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnRuleStatus::Completed => {
                return next == BudgetBurnRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rate - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnRule {
        BudgetBurnRule {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            issuer: self.issuer.clone(),
            rotation_days: self.rotation_days,
            pinned: self.pinned,
            country: self.country.clone(),
            timezone: self.timezone.clone(),
            region: self.region.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a budget burn rule in status planned.
pub fn new_budget_burn_rule(code: &str, name: &str) -> BudgetBurnRule {
    BudgetBurnRule {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        issuer: "".to_string(),
        rotation_days: 0,
        pinned: false,
        country: "".to_string(),
        timezone: "".to_string(),
        region: "".to_string(),
        rate: 0.0,
        status: BudgetBurnRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_budget_burn_rule("code-596", "name-786");
        assert_eq!(item.status, BudgetBurnRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_budget_burn_rule("code-596", "name-786");
        if item.move_to(BudgetBurnRuleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnRuleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_budget_burn_rule("code-596", "name-786");
        item.status = BudgetBurnRuleStatus::Cancelled;
        assert!(
            item.move_to(BudgetBurnRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
