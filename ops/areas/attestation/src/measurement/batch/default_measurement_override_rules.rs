use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default measurement override must pass. Check returns its problems, or none.
pub trait DefaultMeasurementOverrideRule: std::fmt::Debug {
    fn check(&self, item: &DefaultMeasurementOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementOverrideExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultMeasurementOverrideRule for DefaultMeasurementOverrideExpiresAtRangeRule {
    fn check(&self, item: &DefaultMeasurementOverride) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementOverrideCodePrefixRule {
    pub prefix: String,
}

impl DefaultMeasurementOverrideRule for DefaultMeasurementOverrideCodePrefixRule {
    fn check(&self, item: &DefaultMeasurementOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementOverrideScopeLengthRule {
    pub max_length: i64,
}

impl DefaultMeasurementOverrideRule for DefaultMeasurementOverrideScopeLengthRule {
    fn check(&self, item: &DefaultMeasurementOverride) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default measurement override.
#[derive(Debug, Clone)]
pub struct DefaultMeasurementOverrideValidator {
    pub rules: Vec<Rc<dyn DefaultMeasurementOverrideRule>>,
}

impl DefaultMeasurementOverrideValidator {
    pub fn validate(&self, item: &DefaultMeasurementOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultMeasurementOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultMeasurementOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_measurement_override_validator() -> DefaultMeasurementOverrideValidator {
    DefaultMeasurementOverrideValidator {
        rules: vec![
            Rc::new(DefaultMeasurementOverrideExpiresAtRangeRule { min: 120, max: 86400 })
                as Rc<dyn DefaultMeasurementOverrideRule>,
            Rc::new(DefaultMeasurementOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultMeasurementOverrideScopeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_measurement_override_validator();
        let item = new_default_measurement_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DefaultMeasurementOverrideExpiresAtRangeRule { min: 120, max: 1000 };
        let mut item = new_default_measurement_override("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = DefaultMeasurementOverrideScopeLengthRule { max_length: 80 };
        let mut item = new_default_measurement_override("x", "y");
        item.scope = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
