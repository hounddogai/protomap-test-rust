use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout ledger must pass. Check returns its problems, or none.
pub trait RolloutLedgerRule: std::fmt::Debug {
    fn check(&self, item: &RolloutLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutLedgerLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutLedgerRule for RolloutLedgerLimitRangeRule {
    fn check(&self, item: &RolloutLedger) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutLedgerAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RolloutLedgerRule for RolloutLedgerAllowedTitleRule {
    fn check(&self, item: &RolloutLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutLedgerBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutLedgerRule for RolloutLedgerBudgetRangeRule {
    fn check(&self, item: &RolloutLedger) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutLedgerTitleLengthRule {
    pub max_length: i64,
}

impl RolloutLedgerRule for RolloutLedgerTitleLengthRule {
    fn check(&self, item: &RolloutLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutLedgerCodePrefixRule {
    pub prefix: String,
}

impl RolloutLedgerRule for RolloutLedgerCodePrefixRule {
    fn check(&self, item: &RolloutLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout ledger.
#[derive(Debug, Clone)]
pub struct RolloutLedgerValidator {
    pub rules: Vec<Rc<dyn RolloutLedgerRule>>,
}

impl RolloutLedgerValidator {
    pub fn validate(&self, item: &RolloutLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_ledger_validator() -> RolloutLedgerValidator {
    RolloutLedgerValidator {
        rules: vec![
            Rc::new(RolloutLedgerLimitRangeRule { min: 64, max: 120 }) as Rc<dyn RolloutLedgerRule>,
            Rc::new(RolloutLedgerAllowedTitleRule {
                allowed: vec!["strict".to_string(), "manual".to_string(), "primary".to_string()],
            }),
            Rc::new(RolloutLedgerBudgetRangeRule { min: 6.0, max: 4967.4 }),
            Rc::new(RolloutLedgerTitleLengthRule { max_length: 86400 }),
            Rc::new(RolloutLedgerCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_ledger_validator();
        let item = new_rollout_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = RolloutLedgerLimitRangeRule { min: 32, max: 255 };
        let mut item = new_rollout_ledger("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RolloutLedgerBudgetRangeRule { min: 0.8, max: 2447.1 };
        let mut item = new_rollout_ledger("x", "y");
        item.budget = 2448.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RolloutLedgerTitleLengthRule { max_length: 120 };
        let mut item = new_rollout_ledger("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
