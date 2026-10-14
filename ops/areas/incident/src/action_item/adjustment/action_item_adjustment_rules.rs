use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item adjustment must pass. Check returns its problems, or none.
pub trait ActionItemAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAdjustmentAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl ActionItemAdjustmentRule for ActionItemAdjustmentAllowedDescriptionRule {
    fn check(&self, item: &ActionItemAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is duplicated", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAdjustmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemAdjustmentRule for ActionItemAdjustmentCeilingRangeRule {
    fn check(&self, item: &ActionItemAdjustment) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAdjustmentSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemAdjustmentRule for ActionItemAdjustmentSequenceRangeRule {
    fn check(&self, item: &ActionItemAdjustment) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl ActionItemAdjustmentRule for ActionItemAdjustmentNameLengthRule {
    fn check(&self, item: &ActionItemAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item adjustment.
#[derive(Debug, Clone)]
pub struct ActionItemAdjustmentValidator {
    pub rules: Vec<Rc<dyn ActionItemAdjustmentRule>>,
}

impl ActionItemAdjustmentValidator {
    pub fn validate(&self, item: &ActionItemAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_adjustment_validator() -> ActionItemAdjustmentValidator {
    ActionItemAdjustmentValidator {
        rules: vec![
            Rc::new(ActionItemAdjustmentAllowedDescriptionRule { allowed: Vec::new() })
                as Rc<dyn ActionItemAdjustmentRule>,
            Rc::new(ActionItemAdjustmentCeilingRangeRule { min: 3.1, max: 4178.0 }),
            Rc::new(ActionItemAdjustmentSequenceRangeRule { min: 80, max: 255 }),
            Rc::new(ActionItemAdjustmentNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_adjustment_validator();
        let item = new_action_item_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ActionItemAdjustmentCeilingRangeRule { min: 5.4, max: 769.2 };
        let mut item = new_action_item_adjustment("x", "y");
        item.ceiling = 770.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ActionItemAdjustmentSequenceRangeRule { min: 255, max: 1000 };
        let mut item = new_action_item_adjustment("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ActionItemAdjustmentNameLengthRule { max_length: 86400 };
        let mut item = new_action_item_adjustment("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
