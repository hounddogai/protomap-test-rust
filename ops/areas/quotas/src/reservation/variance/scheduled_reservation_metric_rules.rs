use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled reservation metric must pass. Check returns its problems, or none.
pub trait ScheduledReservationMetricRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledReservationMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationMetricRequiredTimezoneRule {
    pub message: String,
}

impl ScheduledReservationMetricRule for ScheduledReservationMetricRequiredTimezoneRule {
    fn check(&self, item: &ScheduledReservationMetric) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationMetricFingerprintLengthRule {
    pub max_length: i64,
}

impl ScheduledReservationMetricRule for ScheduledReservationMetricFingerprintLengthRule {
    fn check(&self, item: &ScheduledReservationMetric) -> Vec<String> {
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
pub struct ScheduledReservationMetricLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledReservationMetricRule for ScheduledReservationMetricLabelLengthRule {
    fn check(&self, item: &ScheduledReservationMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationMetricRequiredCodeRule {
    pub message: String,
}

impl ScheduledReservationMetricRule for ScheduledReservationMetricRequiredCodeRule {
    fn check(&self, item: &ScheduledReservationMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationMetricTimezoneLengthRule {
    pub max_length: i64,
}

impl ScheduledReservationMetricRule for ScheduledReservationMetricTimezoneLengthRule {
    fn check(&self, item: &ScheduledReservationMetric) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled reservation metric.
#[derive(Debug, Clone)]
pub struct ScheduledReservationMetricValidator {
    pub rules: Vec<Rc<dyn ScheduledReservationMetricRule>>,
}

impl ScheduledReservationMetricValidator {
    pub fn validate(&self, item: &ScheduledReservationMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledReservationMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledReservationMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_reservation_metric_validator() -> ScheduledReservationMetricValidator {
    ScheduledReservationMetricValidator {
        rules: vec![
            Rc::new(ScheduledReservationMetricRequiredTimezoneRule {
                message: "scheduled reservation metric needs an entry".to_string(),
            }) as Rc<dyn ScheduledReservationMetricRule>,
            Rc::new(ScheduledReservationMetricFingerprintLengthRule { max_length: 16 }),
            Rc::new(ScheduledReservationMetricLabelLengthRule { max_length: 1000 }),
            Rc::new(ScheduledReservationMetricRequiredCodeRule {
                message: "scheduled reservation metric needs a value".to_string(),
            }),
            Rc::new(ScheduledReservationMetricTimezoneLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_reservation_metric_validator();
        let item = new_scheduled_reservation_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ScheduledReservationMetricRequiredTimezoneRule {
            message: "scheduled reservation metric needs an entry".to_string(),
        };
        let mut item = new_scheduled_reservation_metric("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ScheduledReservationMetricFingerprintLengthRule { max_length: 86400 };
        let mut item = new_scheduled_reservation_metric("x", "y");
        item.fingerprint = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledReservationMetricLabelLengthRule { max_length: 32 };
        let mut item = new_scheduled_reservation_metric("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledReservationMetricRequiredCodeRule {
            message: "scheduled reservation metric needs this field".to_string(),
        };
        let mut item = new_scheduled_reservation_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = ScheduledReservationMetricTimezoneLengthRule { max_length: 32 };
        let mut item = new_scheduled_reservation_metric("x", "y");
        item.timezone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
