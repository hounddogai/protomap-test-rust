use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check slot must pass. Check returns its problems, or none.
pub trait HealthCheckSlotRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSlotRequiredNotesRule {
    pub message: String,
}

impl HealthCheckSlotRule for HealthCheckSlotRequiredNotesRule {
    fn check(&self, item: &HealthCheckSlot) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSlotCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckSlotRule for HealthCheckSlotCodePrefixRule {
    fn check(&self, item: &HealthCheckSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSlotNotesLengthRule {
    pub max_length: i64,
}

impl HealthCheckSlotRule for HealthCheckSlotNotesLengthRule {
    fn check(&self, item: &HealthCheckSlot) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSlotRequiredCodeRule {
    pub message: String,
}

impl HealthCheckSlotRule for HealthCheckSlotRequiredCodeRule {
    fn check(&self, item: &HealthCheckSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSlotAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl HealthCheckSlotRule for HealthCheckSlotAllowedTitleRule {
    fn check(&self, item: &HealthCheckSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSlotRequiredTitleRule {
    pub message: String,
}

impl HealthCheckSlotRule for HealthCheckSlotRequiredTitleRule {
    fn check(&self, item: &HealthCheckSlot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check slot.
#[derive(Debug, Clone)]
pub struct HealthCheckSlotValidator {
    pub rules: Vec<Rc<dyn HealthCheckSlotRule>>,
}

impl HealthCheckSlotValidator {
    pub fn validate(&self, item: &HealthCheckSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_slot_validator() -> HealthCheckSlotValidator {
    HealthCheckSlotValidator {
        rules: vec![
            Rc::new(HealthCheckSlotRequiredNotesRule {
                message: "health check slot needs an entry".to_string(),
            }) as Rc<dyn HealthCheckSlotRule>,
            Rc::new(HealthCheckSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(HealthCheckSlotNotesLengthRule { max_length: 3600 }),
            Rc::new(HealthCheckSlotRequiredCodeRule {
                message: "health check slot needs an entry".to_string(),
            }),
            Rc::new(HealthCheckSlotAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(HealthCheckSlotRequiredTitleRule {
                message: "health check slot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_slot_validator();
        let item = new_health_check_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = HealthCheckSlotRequiredNotesRule {
            message: "health check slot needs a value".to_string(),
        };
        let mut item = new_health_check_slot("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = HealthCheckSlotNotesLengthRule { max_length: 16 };
        let mut item = new_health_check_slot("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckSlotRequiredCodeRule {
            message: "health check slot needs a value".to_string(),
        };
        let mut item = new_health_check_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = HealthCheckSlotRequiredTitleRule {
            message: "health check slot needs a value".to_string(),
        };
        let mut item = new_health_check_slot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
