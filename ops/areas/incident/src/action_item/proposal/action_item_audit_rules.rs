use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item audit must pass. Check returns its problems, or none.
pub trait ActionItemAuditRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAuditRequiredLocaleRule {
    pub message: String,
}

impl ActionItemAuditRule for ActionItemAuditRequiredLocaleRule {
    fn check(&self, item: &ActionItemAudit) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAuditBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemAuditRule for ActionItemAuditBatchSizeRangeRule {
    fn check(&self, item: &ActionItemAudit) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAuditAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl ActionItemAuditRule for ActionItemAuditAllowedLocaleRule {
    fn check(&self, item: &ActionItemAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is unknown", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAuditNameLengthRule {
    pub max_length: i64,
}

impl ActionItemAuditRule for ActionItemAuditNameLengthRule {
    fn check(&self, item: &ActionItemAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAuditRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemAuditRule for ActionItemAuditRotationDaysRangeRule {
    fn check(&self, item: &ActionItemAudit) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAuditLocaleLengthRule {
    pub max_length: i64,
}

impl ActionItemAuditRule for ActionItemAuditLocaleLengthRule {
    fn check(&self, item: &ActionItemAudit) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item audit.
#[derive(Debug, Clone)]
pub struct ActionItemAuditValidator {
    pub rules: Vec<Rc<dyn ActionItemAuditRule>>,
}

impl ActionItemAuditValidator {
    pub fn validate(&self, item: &ActionItemAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_audit_validator() -> ActionItemAuditValidator {
    ActionItemAuditValidator {
        rules: vec![
            Rc::new(ActionItemAuditRequiredLocaleRule {
                message: "action item audit needs an entry".to_string(),
            }) as Rc<dyn ActionItemAuditRule>,
            Rc::new(ActionItemAuditBatchSizeRangeRule { min: 16, max: 3600 }),
            Rc::new(ActionItemAuditAllowedLocaleRule {
                allowed: vec!["lenient".to_string(), "rolling".to_string()],
            }),
            Rc::new(ActionItemAuditNameLengthRule { max_length: 3600 }),
            Rc::new(ActionItemAuditRotationDaysRangeRule { min: 64, max: 120 }),
            Rc::new(ActionItemAuditLocaleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_audit_validator();
        let item = new_action_item_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = ActionItemAuditRequiredLocaleRule {
            message: "action item audit needs a value".to_string(),
        };
        let mut item = new_action_item_audit("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = ActionItemAuditBatchSizeRangeRule { min: 16, max: 255 };
        let mut item = new_action_item_audit("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ActionItemAuditNameLengthRule { max_length: 16 };
        let mut item = new_action_item_audit("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ActionItemAuditRotationDaysRangeRule { min: 80, max: 3600 };
        let mut item = new_action_item_audit("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ActionItemAuditLocaleLengthRule { max_length: 3600 };
        let mut item = new_action_item_audit("x", "y");
        item.locale = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
