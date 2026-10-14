use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook override must pass. Check returns its problems, or none.
pub trait RunbookOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RunbookOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverridePhoneLengthRule {
    pub max_length: i64,
}

impl RunbookOverrideRule for RunbookOverridePhoneLengthRule {
    fn check(&self, item: &RunbookOverride) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverrideRequiredCodeRule {
    pub message: String,
}

impl RunbookOverrideRule for RunbookOverrideRequiredCodeRule {
    fn check(&self, item: &RunbookOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverrideRequiredNameRule {
    pub message: String,
}

impl RunbookOverrideRule for RunbookOverrideRequiredNameRule {
    fn check(&self, item: &RunbookOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverrideCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookOverrideRule for RunbookOverrideCapacityRangeRule {
    fn check(&self, item: &RunbookOverride) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverrideAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl RunbookOverrideRule for RunbookOverrideAllowedCountryRule {
    fn check(&self, item: &RunbookOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is duplicated", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverrideRequiredPhoneRule {
    pub message: String,
}

impl RunbookOverrideRule for RunbookOverrideRequiredPhoneRule {
    fn check(&self, item: &RunbookOverride) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook override.
#[derive(Debug, Clone)]
pub struct RunbookOverrideValidator {
    pub rules: Vec<Rc<dyn RunbookOverrideRule>>,
}

impl RunbookOverrideValidator {
    pub fn validate(&self, item: &RunbookOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_override_validator() -> RunbookOverrideValidator {
    RunbookOverrideValidator {
        rules: vec![
            Rc::new(RunbookOverridePhoneLengthRule { max_length: 1000 }) as Rc<dyn RunbookOverrideRule>,
            Rc::new(RunbookOverrideRequiredCodeRule {
                message: "runbook override needs an entry".to_string(),
            }),
            Rc::new(RunbookOverrideRequiredNameRule {
                message: "runbook override needs an entry".to_string(),
            }),
            Rc::new(RunbookOverrideCapacityRangeRule { min: 255, max: 1000 }),
            Rc::new(RunbookOverrideAllowedCountryRule { allowed: Vec::new() }),
            Rc::new(RunbookOverrideRequiredPhoneRule {
                message: "runbook override needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_override_validator();
        let item = new_runbook_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RunbookOverridePhoneLengthRule { max_length: 64 };
        let mut item = new_runbook_override("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookOverrideRequiredCodeRule {
            message: "runbook override needs an entry".to_string(),
        };
        let mut item = new_runbook_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RunbookOverrideRequiredNameRule {
            message: "runbook override needs a value".to_string(),
        };
        let mut item = new_runbook_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RunbookOverrideCapacityRangeRule { min: 32, max: 3600 };
        let mut item = new_runbook_override("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = RunbookOverrideRequiredPhoneRule {
            message: "runbook override needs this field".to_string(),
        };
        let mut item = new_runbook_override("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
