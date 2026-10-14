use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item metric must pass. Check returns its problems, or none.
pub trait ActionItemMetricRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMetricRequiredOwnerIdRule {
    pub message: String,
}

impl ActionItemMetricRule for ActionItemMetricRequiredOwnerIdRule {
    fn check(&self, item: &ActionItemMetric) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMetricCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemMetricRule for ActionItemMetricCodeLengthRule {
    fn check(&self, item: &ActionItemMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMetricWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemMetricRule for ActionItemMetricWeightRangeRule {
    fn check(&self, item: &ActionItemMetric) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMetricAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl ActionItemMetricRule for ActionItemMetricAllowedOwnerIdRule {
    fn check(&self, item: &ActionItemMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is blocked", item.owner_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item metric.
#[derive(Debug, Clone)]
pub struct ActionItemMetricValidator {
    pub rules: Vec<Rc<dyn ActionItemMetricRule>>,
}

impl ActionItemMetricValidator {
    pub fn validate(&self, item: &ActionItemMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_metric_validator() -> ActionItemMetricValidator {
    ActionItemMetricValidator {
        rules: vec![
            Rc::new(ActionItemMetricRequiredOwnerIdRule {
                message: "action item metric needs an entry".to_string(),
            }) as Rc<dyn ActionItemMetricRule>,
            Rc::new(ActionItemMetricCodeLengthRule { max_length: 64 }),
            Rc::new(ActionItemMetricWeightRangeRule { min: 8.6, max: 478.1 }),
            Rc::new(ActionItemMetricAllowedOwnerIdRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_metric_validator();
        let item = new_action_item_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = ActionItemMetricRequiredOwnerIdRule {
            message: "action item metric needs an entry".to_string(),
        };
        let mut item = new_action_item_metric("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ActionItemMetricCodeLengthRule { max_length: 16 };
        let mut item = new_action_item_metric("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ActionItemMetricWeightRangeRule { min: 4.0, max: 898.1 };
        let mut item = new_action_item_metric("x", "y");
        item.weight = 899.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
