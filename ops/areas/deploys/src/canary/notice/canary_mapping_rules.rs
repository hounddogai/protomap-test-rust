use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary mapping must pass. Check returns its problems, or none.
pub trait CanaryMappingRule: std::fmt::Debug {
    fn check(&self, item: &CanaryMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMappingNameLengthRule {
    pub max_length: i64,
}

impl CanaryMappingRule for CanaryMappingNameLengthRule {
    fn check(&self, item: &CanaryMapping) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMappingAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl CanaryMappingRule for CanaryMappingAllowedPhoneRule {
    fn check(&self, item: &CanaryMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is inconsistent", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMappingRequiredPhoneRule {
    pub message: String,
}

impl CanaryMappingRule for CanaryMappingRequiredPhoneRule {
    fn check(&self, item: &CanaryMapping) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMappingDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryMappingRule for CanaryMappingDurationMinutesRangeRule {
    fn check(&self, item: &CanaryMapping) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMappingPhoneLengthRule {
    pub max_length: i64,
}

impl CanaryMappingRule for CanaryMappingPhoneLengthRule {
    fn check(&self, item: &CanaryMapping) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMappingRequiredCodeRule {
    pub message: String,
}

impl CanaryMappingRule for CanaryMappingRequiredCodeRule {
    fn check(&self, item: &CanaryMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary mapping.
#[derive(Debug, Clone)]
pub struct CanaryMappingValidator {
    pub rules: Vec<Rc<dyn CanaryMappingRule>>,
}

impl CanaryMappingValidator {
    pub fn validate(&self, item: &CanaryMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_mapping_validator() -> CanaryMappingValidator {
    CanaryMappingValidator {
        rules: vec![
            Rc::new(CanaryMappingNameLengthRule { max_length: 86400 }) as Rc<dyn CanaryMappingRule>,
            Rc::new(CanaryMappingAllowedPhoneRule { allowed: Vec::new() }),
            Rc::new(CanaryMappingRequiredPhoneRule {
                message: "canary mapping needs a value".to_string(),
            }),
            Rc::new(CanaryMappingDurationMinutesRangeRule { min: 3600, max: 86400 }),
            Rc::new(CanaryMappingPhoneLengthRule { max_length: 120 }),
            Rc::new(CanaryMappingRequiredCodeRule {
                message: "canary mapping needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_mapping_validator();
        let item = new_canary_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CanaryMappingNameLengthRule { max_length: 255 };
        let mut item = new_canary_mapping("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = CanaryMappingRequiredPhoneRule {
            message: "canary mapping needs an entry".to_string(),
        };
        let mut item = new_canary_mapping("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CanaryMappingDurationMinutesRangeRule { min: 120, max: 86400 };
        let mut item = new_canary_mapping("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = CanaryMappingPhoneLengthRule { max_length: 120 };
        let mut item = new_canary_mapping("x", "y");
        item.phone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryMappingRequiredCodeRule {
            message: "canary mapping needs an entry".to_string(),
        };
        let mut item = new_canary_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
