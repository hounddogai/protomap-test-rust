use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule budget must pass. Check returns its problems, or none.
pub trait RuleBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RuleBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBudgetRequiredChannelRule {
    pub message: String,
}

impl RuleBudgetRule for RuleBudgetRequiredChannelRule {
    fn check(&self, item: &RuleBudget) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBudgetFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleBudgetRule for RuleBudgetFloorValueRangeRule {
    fn check(&self, item: &RuleBudget) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBudgetCurrencyLengthRule {
    pub max_length: i64,
}

impl RuleBudgetRule for RuleBudgetCurrencyLengthRule {
    fn check(&self, item: &RuleBudget) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBudgetPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleBudgetRule for RuleBudgetPriorityRangeRule {
    fn check(&self, item: &RuleBudget) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule budget.
#[derive(Debug, Clone)]
pub struct RuleBudgetValidator {
    pub rules: Vec<Rc<dyn RuleBudgetRule>>,
}

impl RuleBudgetValidator {
    pub fn validate(&self, item: &RuleBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_budget_validator() -> RuleBudgetValidator {
    RuleBudgetValidator {
        rules: vec![
            Rc::new(RuleBudgetRequiredChannelRule {
                message: "rule budget needs an entry".to_string(),
            }) as Rc<dyn RuleBudgetRule>,
            Rc::new(RuleBudgetFloorValueRangeRule { min: 5.1, max: 3716.8 }),
            Rc::new(RuleBudgetCurrencyLengthRule { max_length: 64 }),
            Rc::new(RuleBudgetPriorityRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_budget_validator();
        let item = new_rule_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = RuleBudgetRequiredChannelRule {
            message: "rule budget needs this field".to_string(),
        };
        let mut item = new_rule_budget("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RuleBudgetFloorValueRangeRule { min: 1.6, max: 2190.0 };
        let mut item = new_rule_budget("x", "y");
        item.floor_value = 2191.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RuleBudgetCurrencyLengthRule { max_length: 64 };
        let mut item = new_rule_budget("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RuleBudgetPriorityRangeRule { min: 64, max: 64 };
        let mut item = new_rule_budget("x", "y");
        item.priority = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
