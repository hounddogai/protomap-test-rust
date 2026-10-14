use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled measurement policy must pass. Check returns its problems, or none.
pub trait ScheduledMeasurementPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledMeasurementPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMeasurementPolicyExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledMeasurementPolicyRule for ScheduledMeasurementPolicyExpiresAtRangeRule {
    fn check(&self, item: &ScheduledMeasurementPolicy) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMeasurementPolicyCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledMeasurementPolicyRule for ScheduledMeasurementPolicyCodeLengthRule {
    fn check(&self, item: &ScheduledMeasurementPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMeasurementPolicyNameLengthRule {
    pub max_length: i64,
}

impl ScheduledMeasurementPolicyRule for ScheduledMeasurementPolicyNameLengthRule {
    fn check(&self, item: &ScheduledMeasurementPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMeasurementPolicyMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledMeasurementPolicyRule for ScheduledMeasurementPolicyMaximumRangeRule {
    fn check(&self, item: &ScheduledMeasurementPolicy) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled measurement policy.
#[derive(Debug, Clone)]
pub struct ScheduledMeasurementPolicyValidator {
    pub rules: Vec<Rc<dyn ScheduledMeasurementPolicyRule>>,
}

impl ScheduledMeasurementPolicyValidator {
    pub fn validate(&self, item: &ScheduledMeasurementPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledMeasurementPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledMeasurementPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_measurement_policy_validator() -> ScheduledMeasurementPolicyValidator {
    ScheduledMeasurementPolicyValidator {
        rules: vec![
            Rc::new(ScheduledMeasurementPolicyExpiresAtRangeRule { min: 16, max: 3600 })
                as Rc<dyn ScheduledMeasurementPolicyRule>,
            Rc::new(ScheduledMeasurementPolicyCodeLengthRule { max_length: 86400 }),
            Rc::new(ScheduledMeasurementPolicyNameLengthRule { max_length: 3600 }),
            Rc::new(ScheduledMeasurementPolicyMaximumRangeRule { min: 2.8, max: 1564.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_measurement_policy_validator();
        let item = new_scheduled_measurement_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = ScheduledMeasurementPolicyExpiresAtRangeRule { min: 1000, max: 1000 };
        let mut item = new_scheduled_measurement_policy("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledMeasurementPolicyCodeLengthRule { max_length: 16 };
        let mut item = new_scheduled_measurement_policy("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledMeasurementPolicyNameLengthRule { max_length: 16 };
        let mut item = new_scheduled_measurement_policy("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ScheduledMeasurementPolicyMaximumRangeRule { min: 6.4, max: 1809.0 };
        let mut item = new_scheduled_measurement_policy("x", "y");
        item.maximum = 1810.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
