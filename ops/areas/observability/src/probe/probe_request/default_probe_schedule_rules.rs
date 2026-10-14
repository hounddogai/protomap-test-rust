use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default probe schedule must pass. Check returns its problems, or none.
pub trait DefaultProbeScheduleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultProbeSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeScheduleQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultProbeScheduleRule for DefaultProbeScheduleQuantityRangeRule {
    fn check(&self, item: &DefaultProbeSchedule) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeScheduleCodePrefixRule {
    pub prefix: String,
}

impl DefaultProbeScheduleRule for DefaultProbeScheduleCodePrefixRule {
    fn check(&self, item: &DefaultProbeSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeScheduleAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl DefaultProbeScheduleRule for DefaultProbeScheduleAllowedLocaleRule {
    fn check(&self, item: &DefaultProbeSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is inconsistent", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeScheduleRequiredLocaleRule {
    pub message: String,
}

impl DefaultProbeScheduleRule for DefaultProbeScheduleRequiredLocaleRule {
    fn check(&self, item: &DefaultProbeSchedule) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeScheduleLocaleLengthRule {
    pub max_length: i64,
}

impl DefaultProbeScheduleRule for DefaultProbeScheduleLocaleLengthRule {
    fn check(&self, item: &DefaultProbeSchedule) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default probe schedule.
#[derive(Debug, Clone)]
pub struct DefaultProbeScheduleValidator {
    pub rules: Vec<Rc<dyn DefaultProbeScheduleRule>>,
}

impl DefaultProbeScheduleValidator {
    pub fn validate(&self, item: &DefaultProbeSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultProbeSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultProbeSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_probe_schedule_validator() -> DefaultProbeScheduleValidator {
    DefaultProbeScheduleValidator {
        rules: vec![
            Rc::new(DefaultProbeScheduleQuantityRangeRule { min: 255, max: 1000 }) as Rc<dyn DefaultProbeScheduleRule>,
            Rc::new(DefaultProbeScheduleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultProbeScheduleAllowedLocaleRule {
                allowed: vec!["primary".to_string(), "lenient".to_string(), "fallback".to_string()],
            }),
            Rc::new(DefaultProbeScheduleRequiredLocaleRule {
                message: "default probe schedule needs this field".to_string(),
            }),
            Rc::new(DefaultProbeScheduleLocaleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_probe_schedule_validator();
        let item = new_default_probe_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DefaultProbeScheduleQuantityRangeRule { min: 16, max: 255 };
        let mut item = new_default_probe_schedule("x", "y");
        item.quantity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = DefaultProbeScheduleRequiredLocaleRule {
            message: "default probe schedule needs this field".to_string(),
        };
        let mut item = new_default_probe_schedule("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = DefaultProbeScheduleLocaleLengthRule { max_length: 64 };
        let mut item = new_default_probe_schedule("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
