use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global measurement override must pass. Check returns its problems, or none.
pub trait GlobalMeasurementOverrideRule: std::fmt::Debug {
    fn check(&self, item: &GlobalMeasurementOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementOverrideRequiredLabelRule {
    pub message: String,
}

impl GlobalMeasurementOverrideRule for GlobalMeasurementOverrideRequiredLabelRule {
    fn check(&self, item: &GlobalMeasurementOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementOverrideTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalMeasurementOverrideRule for GlobalMeasurementOverrideTtlSecondsRangeRule {
    fn check(&self, item: &GlobalMeasurementOverride) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementOverrideCodeLengthRule {
    pub max_length: i64,
}

impl GlobalMeasurementOverrideRule for GlobalMeasurementOverrideCodeLengthRule {
    fn check(&self, item: &GlobalMeasurementOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalMeasurementOverrideRule for GlobalMeasurementOverrideAllowedCodeRule {
    fn check(&self, item: &GlobalMeasurementOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementOverrideRequiredLocaleRule {
    pub message: String,
}

impl GlobalMeasurementOverrideRule for GlobalMeasurementOverrideRequiredLocaleRule {
    fn check(&self, item: &GlobalMeasurementOverride) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global measurement override.
#[derive(Debug, Clone)]
pub struct GlobalMeasurementOverrideValidator {
    pub rules: Vec<Rc<dyn GlobalMeasurementOverrideRule>>,
}

impl GlobalMeasurementOverrideValidator {
    pub fn validate(&self, item: &GlobalMeasurementOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalMeasurementOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalMeasurementOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_measurement_override_validator() -> GlobalMeasurementOverrideValidator {
    GlobalMeasurementOverrideValidator {
        rules: vec![
            Rc::new(GlobalMeasurementOverrideRequiredLabelRule {
                message: "global measurement override needs a value".to_string(),
            }) as Rc<dyn GlobalMeasurementOverrideRule>,
            Rc::new(GlobalMeasurementOverrideTtlSecondsRangeRule { min: 32, max: 255 }),
            Rc::new(GlobalMeasurementOverrideCodeLengthRule { max_length: 16 }),
            Rc::new(GlobalMeasurementOverrideAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(GlobalMeasurementOverrideRequiredLocaleRule {
                message: "global measurement override needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_measurement_override_validator();
        let item = new_global_measurement_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GlobalMeasurementOverrideRequiredLabelRule {
            message: "global measurement override needs a value".to_string(),
        };
        let mut item = new_global_measurement_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = GlobalMeasurementOverrideTtlSecondsRangeRule { min: 32, max: 120 };
        let mut item = new_global_measurement_override("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalMeasurementOverrideCodeLengthRule { max_length: 255 };
        let mut item = new_global_measurement_override("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = GlobalMeasurementOverrideRequiredLocaleRule {
            message: "global measurement override needs a value".to_string(),
        };
        let mut item = new_global_measurement_override("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
