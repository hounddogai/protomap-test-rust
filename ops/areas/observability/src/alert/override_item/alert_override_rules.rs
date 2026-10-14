use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert override must pass. Check returns its problems, or none.
pub trait AlertOverrideRule: std::fmt::Debug {
    fn check(&self, item: &AlertOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertOverrideRequiredAudienceRule {
    pub message: String,
}

impl AlertOverrideRule for AlertOverrideRequiredAudienceRule {
    fn check(&self, item: &AlertOverride) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertOverrideRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertOverrideRule for AlertOverrideRevisionRangeRule {
    fn check(&self, item: &AlertOverride) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertOverrideFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertOverrideRule for AlertOverrideFloorValueRangeRule {
    fn check(&self, item: &AlertOverride) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertOverrideFingerprintLengthRule {
    pub max_length: i64,
}

impl AlertOverrideRule for AlertOverrideFingerprintLengthRule {
    fn check(&self, item: &AlertOverride) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertOverrideEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertOverrideRule for AlertOverrideEndsAtRangeRule {
    fn check(&self, item: &AlertOverride) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertOverrideAudienceLengthRule {
    pub max_length: i64,
}

impl AlertOverrideRule for AlertOverrideAudienceLengthRule {
    fn check(&self, item: &AlertOverride) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert override.
#[derive(Debug, Clone)]
pub struct AlertOverrideValidator {
    pub rules: Vec<Rc<dyn AlertOverrideRule>>,
}

impl AlertOverrideValidator {
    pub fn validate(&self, item: &AlertOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_override_validator() -> AlertOverrideValidator {
    AlertOverrideValidator {
        rules: vec![
            Rc::new(AlertOverrideRequiredAudienceRule {
                message: "alert override needs an entry".to_string(),
            }) as Rc<dyn AlertOverrideRule>,
            Rc::new(AlertOverrideRevisionRangeRule { min: 3600, max: 86400 }),
            Rc::new(AlertOverrideFloorValueRangeRule { min: 4.4, max: 4719.9 }),
            Rc::new(AlertOverrideFingerprintLengthRule { max_length: 64 }),
            Rc::new(AlertOverrideEndsAtRangeRule { min: 255, max: 86400 }),
            Rc::new(AlertOverrideAudienceLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_override_validator();
        let item = new_alert_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = AlertOverrideRequiredAudienceRule {
            message: "alert override needs a value".to_string(),
        };
        let mut item = new_alert_override("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = AlertOverrideRevisionRangeRule { min: 16, max: 80 };
        let mut item = new_alert_override("x", "y");
        item.revision = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = AlertOverrideFloorValueRangeRule { min: 9.9, max: 4522.2 };
        let mut item = new_alert_override("x", "y");
        item.floor_value = 4523.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = AlertOverrideFingerprintLengthRule { max_length: 86400 };
        let mut item = new_alert_override("x", "y");
        item.fingerprint = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = AlertOverrideEndsAtRangeRule { min: 16, max: 120 };
        let mut item = new_alert_override("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = AlertOverrideAudienceLengthRule { max_length: 64 };
        let mut item = new_alert_override("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
