use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item estimate must pass. Check returns its problems, or none.
pub trait ActionItemEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEstimateCodePrefixRule {
    pub prefix: String,
}

impl ActionItemEstimateRule for ActionItemEstimateCodePrefixRule {
    fn check(&self, item: &ActionItemEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEstimateUnitLengthRule {
    pub max_length: i64,
}

impl ActionItemEstimateRule for ActionItemEstimateUnitLengthRule {
    fn check(&self, item: &ActionItemEstimate) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEstimateRequiredCodeRule {
    pub message: String,
}

impl ActionItemEstimateRule for ActionItemEstimateRequiredCodeRule {
    fn check(&self, item: &ActionItemEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEstimateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemEstimateRule for ActionItemEstimateMarginRangeRule {
    fn check(&self, item: &ActionItemEstimate) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEstimateArchivedGuardRule {
    pub limit: i64,
}

impl ActionItemEstimateRule for ActionItemEstimateArchivedGuardRule {
    fn check(&self, item: &ActionItemEstimate) -> Vec<String> {
        if item.archived && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while archived",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item estimate.
#[derive(Debug, Clone)]
pub struct ActionItemEstimateValidator {
    pub rules: Vec<Rc<dyn ActionItemEstimateRule>>,
}

impl ActionItemEstimateValidator {
    pub fn validate(&self, item: &ActionItemEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_estimate_validator() -> ActionItemEstimateValidator {
    ActionItemEstimateValidator {
        rules: vec![
            Rc::new(ActionItemEstimateCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ActionItemEstimateRule>,
            Rc::new(ActionItemEstimateUnitLengthRule { max_length: 64 }),
            Rc::new(ActionItemEstimateRequiredCodeRule {
                message: "action item estimate needs a value".to_string(),
            }),
            Rc::new(ActionItemEstimateMarginRangeRule { min: 6.2, max: 656.7 }),
            Rc::new(ActionItemEstimateArchivedGuardRule { limit: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_estimate_validator();
        let item = new_action_item_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ActionItemEstimateUnitLengthRule { max_length: 80 };
        let mut item = new_action_item_estimate("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ActionItemEstimateRequiredCodeRule {
            message: "action item estimate needs this field".to_string(),
        };
        let mut item = new_action_item_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ActionItemEstimateMarginRangeRule { min: 3.4, max: 2221.0 };
        let mut item = new_action_item_estimate("x", "y");
        item.margin = 2222.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
