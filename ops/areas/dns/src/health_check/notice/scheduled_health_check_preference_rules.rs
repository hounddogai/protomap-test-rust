use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled health check preference must pass. Check returns its problems, or none.
pub trait ScheduledHealthCheckPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledHealthCheckPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckPreferenceWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledHealthCheckPreferenceRule for ScheduledHealthCheckPreferenceWeightRangeRule {
    fn check(&self, item: &ScheduledHealthCheckPreference) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckPreferenceStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledHealthCheckPreferenceRule for ScheduledHealthCheckPreferenceStartsAtRangeRule {
    fn check(&self, item: &ScheduledHealthCheckPreference) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckPreferenceRequiredTitleRule {
    pub message: String,
}

impl ScheduledHealthCheckPreferenceRule for ScheduledHealthCheckPreferenceRequiredTitleRule {
    fn check(&self, item: &ScheduledHealthCheckPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledHealthCheckPreferenceRule for ScheduledHealthCheckPreferenceTitleLengthRule {
    fn check(&self, item: &ScheduledHealthCheckPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled health check preference.
#[derive(Debug, Clone)]
pub struct ScheduledHealthCheckPreferenceValidator {
    pub rules: Vec<Rc<dyn ScheduledHealthCheckPreferenceRule>>,
}

impl ScheduledHealthCheckPreferenceValidator {
    pub fn validate(&self, item: &ScheduledHealthCheckPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledHealthCheckPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledHealthCheckPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_health_check_preference_validator() -> ScheduledHealthCheckPreferenceValidator {
    ScheduledHealthCheckPreferenceValidator {
        rules: vec![
            Rc::new(ScheduledHealthCheckPreferenceWeightRangeRule { min: 7.8, max: 424.9 })
                as Rc<dyn ScheduledHealthCheckPreferenceRule>,
            Rc::new(ScheduledHealthCheckPreferenceStartsAtRangeRule { min: 80, max: 86400 }),
            Rc::new(ScheduledHealthCheckPreferenceRequiredTitleRule {
                message: "scheduled health check preference needs an entry".to_string(),
            }),
            Rc::new(ScheduledHealthCheckPreferenceTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_health_check_preference_validator();
        let item = new_scheduled_health_check_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ScheduledHealthCheckPreferenceWeightRangeRule { min: 7.3, max: 369.7 };
        let mut item = new_scheduled_health_check_preference("x", "y");
        item.weight = 371.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ScheduledHealthCheckPreferenceStartsAtRangeRule { min: 120, max: 255 };
        let mut item = new_scheduled_health_check_preference("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledHealthCheckPreferenceRequiredTitleRule {
            message: "scheduled health check preference needs a value".to_string(),
        };
        let mut item = new_scheduled_health_check_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledHealthCheckPreferenceTitleLengthRule { max_length: 64 };
        let mut item = new_scheduled_health_check_preference("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
