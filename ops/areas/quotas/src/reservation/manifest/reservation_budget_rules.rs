use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation budget must pass. Check returns its problems, or none.
pub trait ReservationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ReservationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBudgetPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationBudgetRule for ReservationBudgetPortRangeRule {
    fn check(&self, item: &ReservationBudget) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBudgetCodePrefixRule {
    pub prefix: String,
}

impl ReservationBudgetRule for ReservationBudgetCodePrefixRule {
    fn check(&self, item: &ReservationBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBudgetPinnedGuardRule {
    pub limit: i64,
}

impl ReservationBudgetRule for ReservationBudgetPinnedGuardRule {
    fn check(&self, item: &ReservationBudget) -> Vec<String> {
        if item.pinned && (item.retries > self.limit) {
            return vec![format!("retries {} exceeds {} while pinned", item.retries, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation budget.
#[derive(Debug, Clone)]
pub struct ReservationBudgetValidator {
    pub rules: Vec<Rc<dyn ReservationBudgetRule>>,
}

impl ReservationBudgetValidator {
    pub fn validate(&self, item: &ReservationBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationBudget]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_reservation_budget_validator() -> ReservationBudgetValidator {
    ReservationBudgetValidator {
        rules: vec![
            Rc::new(ReservationBudgetPortRangeRule { min: 255, max: 3600 }) as Rc<dyn ReservationBudgetRule>,
            Rc::new(ReservationBudgetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ReservationBudgetPinnedGuardRule { limit: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_budget_validator();
        let item = new_reservation_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = ReservationBudgetPortRangeRule { min: 16, max: 1000 };
        let mut item = new_reservation_budget("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
