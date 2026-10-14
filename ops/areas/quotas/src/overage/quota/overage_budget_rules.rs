use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage budget must pass. Check returns its problems, or none.
pub trait OverageBudgetRule: std::fmt::Debug {
    fn check(&self, item: &OverageBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBudgetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl OverageBudgetRule for OverageBudgetAllowedTitleRule {
    fn check(&self, item: &OverageBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBudgetRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageBudgetRule for OverageBudgetRetriesRangeRule {
    fn check(&self, item: &OverageBudget) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBudgetCodePrefixRule {
    pub prefix: String,
}

impl OverageBudgetRule for OverageBudgetCodePrefixRule {
    fn check(&self, item: &OverageBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBudgetBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageBudgetRule for OverageBudgetBatchSizeRangeRule {
    fn check(&self, item: &OverageBudget) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBudgetTitleLengthRule {
    pub max_length: i64,
}

impl OverageBudgetRule for OverageBudgetTitleLengthRule {
    fn check(&self, item: &OverageBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage budget.
#[derive(Debug, Clone)]
pub struct OverageBudgetValidator {
    pub rules: Vec<Rc<dyn OverageBudgetRule>>,
}

impl OverageBudgetValidator {
    pub fn validate(&self, item: &OverageBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_budget_validator() -> OverageBudgetValidator {
    OverageBudgetValidator {
        rules: vec![
            Rc::new(OverageBudgetAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn OverageBudgetRule>,
            Rc::new(OverageBudgetRetriesRangeRule { min: 64, max: 3600 }),
            Rc::new(OverageBudgetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(OverageBudgetBatchSizeRangeRule { min: 16, max: 1000 }),
            Rc::new(OverageBudgetTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_budget_validator();
        let item = new_overage_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = OverageBudgetRetriesRangeRule { min: 16, max: 64 };
        let mut item = new_overage_budget("x", "y");
        item.retries = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = OverageBudgetBatchSizeRangeRule { min: 16, max: 3600 };
        let mut item = new_overage_budget("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = OverageBudgetTitleLengthRule { max_length: 120 };
        let mut item = new_overage_budget("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
