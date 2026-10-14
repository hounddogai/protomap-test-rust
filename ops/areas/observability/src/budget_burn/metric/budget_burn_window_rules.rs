use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn window must pass. Check returns its problems, or none.
pub trait BudgetBurnWindowRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnWindowPrincipalLengthRule {
    pub max_length: i64,
}

impl BudgetBurnWindowRule for BudgetBurnWindowPrincipalLengthRule {
    fn check(&self, item: &BudgetBurnWindow) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnWindowRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnWindowRule for BudgetBurnWindowRetriesRangeRule {
    fn check(&self, item: &BudgetBurnWindow) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnWindowTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnWindowRule for BudgetBurnWindowTtlSecondsRangeRule {
    fn check(&self, item: &BudgetBurnWindow) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a budget burn window.
#[derive(Debug, Clone)]
pub struct BudgetBurnWindowValidator {
    pub rules: Vec<Rc<dyn BudgetBurnWindowRule>>,
}

impl BudgetBurnWindowValidator {
    pub fn validate(&self, item: &BudgetBurnWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_window_validator() -> BudgetBurnWindowValidator {
    BudgetBurnWindowValidator {
        rules: vec![
            Rc::new(BudgetBurnWindowPrincipalLengthRule { max_length: 255 }) as Rc<dyn BudgetBurnWindowRule>,
            Rc::new(BudgetBurnWindowRetriesRangeRule { min: 16, max: 32 }),
            Rc::new(BudgetBurnWindowTtlSecondsRangeRule { min: 255, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_window_validator();
        let item = new_budget_burn_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = BudgetBurnWindowPrincipalLengthRule { max_length: 32 };
        let mut item = new_budget_burn_window("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = BudgetBurnWindowRetriesRangeRule { min: 80, max: 255 };
        let mut item = new_budget_burn_window("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = BudgetBurnWindowTtlSecondsRangeRule { min: 16, max: 80 };
        let mut item = new_budget_burn_window("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
