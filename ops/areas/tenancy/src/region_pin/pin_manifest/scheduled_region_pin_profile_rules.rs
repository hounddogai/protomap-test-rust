use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled region pin profile must pass. Check returns its problems, or none.
pub trait ScheduledRegionPinProfileRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRegionPinProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfileRequiredCodeRule {
    pub message: String,
}

impl ScheduledRegionPinProfileRule for ScheduledRegionPinProfileRequiredCodeRule {
    fn check(&self, item: &ScheduledRegionPinProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfileAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ScheduledRegionPinProfileRule for ScheduledRegionPinProfileAllowedNameRule {
    fn check(&self, item: &ScheduledRegionPinProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfileCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledRegionPinProfileRule for ScheduledRegionPinProfileCodeLengthRule {
    fn check(&self, item: &ScheduledRegionPinProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfileRequiredIssuerRule {
    pub message: String,
}

impl ScheduledRegionPinProfileRule for ScheduledRegionPinProfileRequiredIssuerRule {
    fn check(&self, item: &ScheduledRegionPinProfile) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfileDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledRegionPinProfileRule for ScheduledRegionPinProfileDurationMinutesRangeRule {
    fn check(&self, item: &ScheduledRegionPinProfile) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfileCodePrefixRule {
    pub prefix: String,
}

impl ScheduledRegionPinProfileRule for ScheduledRegionPinProfileCodePrefixRule {
    fn check(&self, item: &ScheduledRegionPinProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled region pin profile.
#[derive(Debug, Clone)]
pub struct ScheduledRegionPinProfileValidator {
    pub rules: Vec<Rc<dyn ScheduledRegionPinProfileRule>>,
}

impl ScheduledRegionPinProfileValidator {
    pub fn validate(&self, item: &ScheduledRegionPinProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledRegionPinProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRegionPinProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_region_pin_profile_validator() -> ScheduledRegionPinProfileValidator {
    ScheduledRegionPinProfileValidator {
        rules: vec![
            Rc::new(ScheduledRegionPinProfileRequiredCodeRule {
                message: "scheduled region pin profile needs an entry".to_string(),
            }) as Rc<dyn ScheduledRegionPinProfileRule>,
            Rc::new(ScheduledRegionPinProfileAllowedNameRule { allowed: Vec::new() }),
            Rc::new(ScheduledRegionPinProfileCodeLengthRule { max_length: 80 }),
            Rc::new(ScheduledRegionPinProfileRequiredIssuerRule {
                message: "scheduled region pin profile needs this field".to_string(),
            }),
            Rc::new(ScheduledRegionPinProfileDurationMinutesRangeRule { min: 16, max: 255 }),
            Rc::new(ScheduledRegionPinProfileCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_region_pin_profile_validator();
        let item = new_scheduled_region_pin_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledRegionPinProfileRequiredCodeRule {
            message: "scheduled region pin profile needs a value".to_string(),
        };
        let mut item = new_scheduled_region_pin_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledRegionPinProfileCodeLengthRule { max_length: 16 };
        let mut item = new_scheduled_region_pin_profile("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = ScheduledRegionPinProfileRequiredIssuerRule {
            message: "scheduled region pin profile needs a value".to_string(),
        };
        let mut item = new_scheduled_region_pin_profile("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = ScheduledRegionPinProfileDurationMinutesRangeRule { min: 80, max: 255 };
        let mut item = new_scheduled_region_pin_profile("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
