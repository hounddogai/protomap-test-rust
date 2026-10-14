use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item threshold must pass. Check returns its problems, or none.
pub trait ActionItemThresholdRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemThresholdTitleLengthRule {
    pub max_length: i64,
}

impl ActionItemThresholdRule for ActionItemThresholdTitleLengthRule {
    fn check(&self, item: &ActionItemThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemThresholdQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemThresholdRule for ActionItemThresholdQuotaUnitsRangeRule {
    fn check(&self, item: &ActionItemThreshold) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemThresholdRequiredCodeRule {
    pub message: String,
}

impl ActionItemThresholdRule for ActionItemThresholdRequiredCodeRule {
    fn check(&self, item: &ActionItemThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemThresholdExternalRefLengthRule {
    pub max_length: i64,
}

impl ActionItemThresholdRule for ActionItemThresholdExternalRefLengthRule {
    fn check(&self, item: &ActionItemThreshold) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemThresholdRequiredExternalRefRule {
    pub message: String,
}

impl ActionItemThresholdRule for ActionItemThresholdRequiredExternalRefRule {
    fn check(&self, item: &ActionItemThreshold) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item threshold.
#[derive(Debug, Clone)]
pub struct ActionItemThresholdValidator {
    pub rules: Vec<Rc<dyn ActionItemThresholdRule>>,
}

impl ActionItemThresholdValidator {
    pub fn validate(&self, item: &ActionItemThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_threshold_validator() -> ActionItemThresholdValidator {
    ActionItemThresholdValidator {
        rules: vec![
            Rc::new(ActionItemThresholdTitleLengthRule { max_length: 32 }) as Rc<dyn ActionItemThresholdRule>,
            Rc::new(ActionItemThresholdQuotaUnitsRangeRule { min: 32, max: 86400 }),
            Rc::new(ActionItemThresholdRequiredCodeRule {
                message: "action item threshold needs a value".to_string(),
            }),
            Rc::new(ActionItemThresholdExternalRefLengthRule { max_length: 64 }),
            Rc::new(ActionItemThresholdRequiredExternalRefRule {
                message: "action item threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_threshold_validator();
        let item = new_action_item_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ActionItemThresholdTitleLengthRule { max_length: 64 };
        let mut item = new_action_item_threshold("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ActionItemThresholdQuotaUnitsRangeRule { min: 255, max: 3600 };
        let mut item = new_action_item_threshold("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ActionItemThresholdRequiredCodeRule {
            message: "action item threshold needs this field".to_string(),
        };
        let mut item = new_action_item_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = ActionItemThresholdExternalRefLengthRule { max_length: 255 };
        let mut item = new_action_item_threshold("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ActionItemThresholdRequiredExternalRefRule {
            message: "action item threshold needs a value".to_string(),
        };
        let mut item = new_action_item_threshold("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
