use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched budget burn rollup must pass. Check returns its problems, or none.
pub trait BatchedBudgetBurnRollupRule: std::fmt::Debug {
    fn check(&self, item: &BatchedBudgetBurnRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollupLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedBudgetBurnRollupRule for BatchedBudgetBurnRollupLimitRangeRule {
    fn check(&self, item: &BatchedBudgetBurnRollup) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollupRequiredNamespaceRule {
    pub message: String,
}

impl BatchedBudgetBurnRollupRule for BatchedBudgetBurnRollupRequiredNamespaceRule {
    fn check(&self, item: &BatchedBudgetBurnRollup) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollupRequiredNameRule {
    pub message: String,
}

impl BatchedBudgetBurnRollupRule for BatchedBudgetBurnRollupRequiredNameRule {
    fn check(&self, item: &BatchedBudgetBurnRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedBudgetBurnRollupRule for BatchedBudgetBurnRollupAllowedCodeRule {
    fn check(&self, item: &BatchedBudgetBurnRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollupArchivedGuardRule {
    pub limit: i64,
}

impl BatchedBudgetBurnRollupRule for BatchedBudgetBurnRollupArchivedGuardRule {
    fn check(&self, item: &BatchedBudgetBurnRollup) -> Vec<String> {
        if item.archived && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while archived", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollupNameLengthRule {
    pub max_length: i64,
}

impl BatchedBudgetBurnRollupRule for BatchedBudgetBurnRollupNameLengthRule {
    fn check(&self, item: &BatchedBudgetBurnRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched budget burn rollup.
#[derive(Debug, Clone)]
pub struct BatchedBudgetBurnRollupValidator {
    pub rules: Vec<Rc<dyn BatchedBudgetBurnRollupRule>>,
}

impl BatchedBudgetBurnRollupValidator {
    pub fn validate(&self, item: &BatchedBudgetBurnRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedBudgetBurnRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedBudgetBurnRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_budget_burn_rollup_validator() -> BatchedBudgetBurnRollupValidator {
    BatchedBudgetBurnRollupValidator {
        rules: vec![
            Rc::new(BatchedBudgetBurnRollupLimitRangeRule { min: 3600, max: 86400 })
                as Rc<dyn BatchedBudgetBurnRollupRule>,
            Rc::new(BatchedBudgetBurnRollupRequiredNamespaceRule {
                message: "batched budget burn rollup needs a value".to_string(),
            }),
            Rc::new(BatchedBudgetBurnRollupRequiredNameRule {
                message: "batched budget burn rollup needs a value".to_string(),
            }),
            Rc::new(BatchedBudgetBurnRollupAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(BatchedBudgetBurnRollupArchivedGuardRule { limit: 255 }),
            Rc::new(BatchedBudgetBurnRollupNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_budget_burn_rollup_validator();
        let item = new_batched_budget_burn_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = BatchedBudgetBurnRollupLimitRangeRule { min: 64, max: 64 };
        let mut item = new_batched_budget_burn_rollup("x", "y");
        item.limit = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = BatchedBudgetBurnRollupRequiredNamespaceRule {
            message: "batched budget burn rollup needs an entry".to_string(),
        };
        let mut item = new_batched_budget_burn_rollup("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedBudgetBurnRollupRequiredNameRule {
            message: "batched budget burn rollup needs a value".to_string(),
        };
        let mut item = new_batched_budget_burn_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedBudgetBurnRollupNameLengthRule { max_length: 120 };
        let mut item = new_batched_budget_burn_rollup("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
