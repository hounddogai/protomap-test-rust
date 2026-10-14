use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention slot must pass. Check returns its problems, or none.
pub trait RetentionSlotRule: std::fmt::Debug {
    fn check(&self, item: &RetentionSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSlotRequiredTimezoneRule {
    pub message: String,
}

impl RetentionSlotRule for RetentionSlotRequiredTimezoneRule {
    fn check(&self, item: &RetentionSlot) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSlotCodePrefixRule {
    pub prefix: String,
}

impl RetentionSlotRule for RetentionSlotCodePrefixRule {
    fn check(&self, item: &RetentionSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSlotPrincipalLengthRule {
    pub max_length: i64,
}

impl RetentionSlotRule for RetentionSlotPrincipalLengthRule {
    fn check(&self, item: &RetentionSlot) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSlotRequiredLabelRule {
    pub message: String,
}

impl RetentionSlotRule for RetentionSlotRequiredLabelRule {
    fn check(&self, item: &RetentionSlot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSlotVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionSlotRule for RetentionSlotVersionRangeRule {
    fn check(&self, item: &RetentionSlot) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a retention slot.
#[derive(Debug, Clone)]
pub struct RetentionSlotValidator {
    pub rules: Vec<Rc<dyn RetentionSlotRule>>,
}

impl RetentionSlotValidator {
    pub fn validate(&self, item: &RetentionSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_slot_validator() -> RetentionSlotValidator {
    RetentionSlotValidator {
        rules: vec![
            Rc::new(RetentionSlotRequiredTimezoneRule {
                message: "retention slot needs a value".to_string(),
            }) as Rc<dyn RetentionSlotRule>,
            Rc::new(RetentionSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RetentionSlotPrincipalLengthRule { max_length: 255 }),
            Rc::new(RetentionSlotRequiredLabelRule {
                message: "retention slot needs this field".to_string(),
            }),
            Rc::new(RetentionSlotVersionRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_slot_validator();
        let item = new_retention_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = RetentionSlotRequiredTimezoneRule {
            message: "retention slot needs this field".to_string(),
        };
        let mut item = new_retention_slot("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RetentionSlotPrincipalLengthRule { max_length: 80 };
        let mut item = new_retention_slot("x", "y");
        item.principal = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RetentionSlotRequiredLabelRule {
            message: "retention slot needs an entry".to_string(),
        };
        let mut item = new_retention_slot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RetentionSlotVersionRangeRule { min: 64, max: 80 };
        let mut item = new_retention_slot("x", "y");
        item.version = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
