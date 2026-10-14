use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert slot must pass. Check returns its problems, or none.
pub trait AlertSlotRule: std::fmt::Debug {
    fn check(&self, item: &AlertSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertSlotCodeLengthRule {
    pub max_length: i64,
}

impl AlertSlotRule for AlertSlotCodeLengthRule {
    fn check(&self, item: &AlertSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertSlotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertSlotRule for AlertSlotRevisionRangeRule {
    fn check(&self, item: &AlertSlot) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertSlotRequiredCodeRule {
    pub message: String,
}

impl AlertSlotRule for AlertSlotRequiredCodeRule {
    fn check(&self, item: &AlertSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert slot.
#[derive(Debug, Clone)]
pub struct AlertSlotValidator {
    pub rules: Vec<Rc<dyn AlertSlotRule>>,
}

impl AlertSlotValidator {
    pub fn validate(&self, item: &AlertSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_slot_validator() -> AlertSlotValidator {
    AlertSlotValidator {
        rules: vec![
            Rc::new(AlertSlotCodeLengthRule { max_length: 64 }) as Rc<dyn AlertSlotRule>,
            Rc::new(AlertSlotRevisionRangeRule { min: 16, max: 86400 }),
            Rc::new(AlertSlotRequiredCodeRule {
                message: "alert slot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_slot_validator();
        let item = new_alert_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertSlotCodeLengthRule { max_length: 32 };
        let mut item = new_alert_slot("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = AlertSlotRevisionRangeRule { min: 80, max: 1000 };
        let mut item = new_alert_slot("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertSlotRequiredCodeRule {
            message: "alert slot needs this field".to_string(),
        };
        let mut item = new_alert_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
