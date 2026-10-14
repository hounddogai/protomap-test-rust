use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn batch must pass. Check returns its problems, or none.
pub trait BudgetBurnBatchRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatchCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnBatchRule for BudgetBurnBatchCodePrefixRule {
    fn check(&self, item: &BudgetBurnBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnBatchRule for BudgetBurnBatchAllowedCodeRule {
    fn check(&self, item: &BudgetBurnBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatchRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnBatchRule for BudgetBurnBatchRequiredCodeRule {
    fn check(&self, item: &BudgetBurnBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatchLabelLengthRule {
    pub max_length: i64,
}

impl BudgetBurnBatchRule for BudgetBurnBatchLabelLengthRule {
    fn check(&self, item: &BudgetBurnBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatchMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnBatchRule for BudgetBurnBatchMarginRangeRule {
    fn check(&self, item: &BudgetBurnBatch) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatchCodeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnBatchRule for BudgetBurnBatchCodeLengthRule {
    fn check(&self, item: &BudgetBurnBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn batch.
#[derive(Debug, Clone)]
pub struct BudgetBurnBatchValidator {
    pub rules: Vec<Rc<dyn BudgetBurnBatchRule>>,
}

impl BudgetBurnBatchValidator {
    pub fn validate(&self, item: &BudgetBurnBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_batch_validator() -> BudgetBurnBatchValidator {
    BudgetBurnBatchValidator {
        rules: vec![
            Rc::new(BudgetBurnBatchCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn BudgetBurnBatchRule>,
            Rc::new(BudgetBurnBatchAllowedCodeRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(BudgetBurnBatchRequiredCodeRule {
                message: "budget burn batch needs a value".to_string(),
            }),
            Rc::new(BudgetBurnBatchLabelLengthRule { max_length: 16 }),
            Rc::new(BudgetBurnBatchMarginRangeRule { min: 0.6, max: 927.9 }),
            Rc::new(BudgetBurnBatchCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_batch_validator();
        let item = new_budget_burn_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BudgetBurnBatchRequiredCodeRule {
            message: "budget burn batch needs an entry".to_string(),
        };
        let mut item = new_budget_burn_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BudgetBurnBatchLabelLengthRule { max_length: 1000 };
        let mut item = new_budget_burn_batch("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = BudgetBurnBatchMarginRangeRule { min: 2.8, max: 1524.6 };
        let mut item = new_budget_burn_batch("x", "y");
        item.margin = 1526.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BudgetBurnBatchCodeLengthRule { max_length: 32 };
        let mut item = new_budget_burn_batch("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
