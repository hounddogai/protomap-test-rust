use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn item must pass. Check returns its problems, or none.
pub trait BudgetBurnItemRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnItemRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnItemRule for BudgetBurnItemRequiredCodeRule {
    fn check(&self, item: &BudgetBurnItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnItemLabelLengthRule {
    pub max_length: i64,
}

impl BudgetBurnItemRule for BudgetBurnItemLabelLengthRule {
    fn check(&self, item: &BudgetBurnItem) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnItemTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnItemRule for BudgetBurnItemTtlSecondsRangeRule {
    fn check(&self, item: &BudgetBurnItem) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnItemRequiredLabelRule {
    pub message: String,
}

impl BudgetBurnItemRule for BudgetBurnItemRequiredLabelRule {
    fn check(&self, item: &BudgetBurnItem) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn item.
#[derive(Debug, Clone)]
pub struct BudgetBurnItemValidator {
    pub rules: Vec<Rc<dyn BudgetBurnItemRule>>,
}

impl BudgetBurnItemValidator {
    pub fn validate(&self, item: &BudgetBurnItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_item_validator() -> BudgetBurnItemValidator {
    BudgetBurnItemValidator {
        rules: vec![
            Rc::new(BudgetBurnItemRequiredCodeRule {
                message: "budget burn item needs an entry".to_string(),
            }) as Rc<dyn BudgetBurnItemRule>,
            Rc::new(BudgetBurnItemLabelLengthRule { max_length: 64 }),
            Rc::new(BudgetBurnItemTtlSecondsRangeRule { min: 16, max: 64 }),
            Rc::new(BudgetBurnItemRequiredLabelRule {
                message: "budget burn item needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_item_validator();
        let item = new_budget_burn_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BudgetBurnItemRequiredCodeRule {
            message: "budget burn item needs an entry".to_string(),
        };
        let mut item = new_budget_burn_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BudgetBurnItemLabelLengthRule { max_length: 16 };
        let mut item = new_budget_burn_item("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = BudgetBurnItemTtlSecondsRangeRule { min: 64, max: 86400 };
        let mut item = new_budget_burn_item("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BudgetBurnItemRequiredLabelRule {
            message: "budget burn item needs an entry".to_string(),
        };
        let mut item = new_budget_burn_item("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
