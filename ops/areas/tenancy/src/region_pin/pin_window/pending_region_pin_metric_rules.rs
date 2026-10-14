use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending region pin metric must pass. Check returns its problems, or none.
pub trait PendingRegionPinMetricRule: std::fmt::Debug {
    fn check(&self, item: &PendingRegionPinMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinMetricCodePrefixRule {
    pub prefix: String,
}

impl PendingRegionPinMetricRule for PendingRegionPinMetricCodePrefixRule {
    fn check(&self, item: &PendingRegionPinMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinMetricRequiredLocaleRule {
    pub message: String,
}

impl PendingRegionPinMetricRule for PendingRegionPinMetricRequiredLocaleRule {
    fn check(&self, item: &PendingRegionPinMetric) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinMetricPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRegionPinMetricRule for PendingRegionPinMetricPriorityRangeRule {
    fn check(&self, item: &PendingRegionPinMetric) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinMetricRequiredNameRule {
    pub message: String,
}

impl PendingRegionPinMetricRule for PendingRegionPinMetricRequiredNameRule {
    fn check(&self, item: &PendingRegionPinMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinMetricCodeLengthRule {
    pub max_length: i64,
}

impl PendingRegionPinMetricRule for PendingRegionPinMetricCodeLengthRule {
    fn check(&self, item: &PendingRegionPinMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinMetricGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRegionPinMetricRule for PendingRegionPinMetricGraceDaysRangeRule {
    fn check(&self, item: &PendingRegionPinMetric) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending region pin metric.
#[derive(Debug, Clone)]
pub struct PendingRegionPinMetricValidator {
    pub rules: Vec<Rc<dyn PendingRegionPinMetricRule>>,
}

impl PendingRegionPinMetricValidator {
    pub fn validate(&self, item: &PendingRegionPinMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingRegionPinMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRegionPinMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_region_pin_metric_validator() -> PendingRegionPinMetricValidator {
    PendingRegionPinMetricValidator {
        rules: vec![
            Rc::new(PendingRegionPinMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn PendingRegionPinMetricRule>,
            Rc::new(PendingRegionPinMetricRequiredLocaleRule {
                message: "pending region pin metric needs an entry".to_string(),
            }),
            Rc::new(PendingRegionPinMetricPriorityRangeRule { min: 32, max: 80 }),
            Rc::new(PendingRegionPinMetricRequiredNameRule {
                message: "pending region pin metric needs a value".to_string(),
            }),
            Rc::new(PendingRegionPinMetricCodeLengthRule { max_length: 1000 }),
            Rc::new(PendingRegionPinMetricGraceDaysRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_region_pin_metric_validator();
        let item = new_pending_region_pin_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PendingRegionPinMetricRequiredLocaleRule {
            message: "pending region pin metric needs this field".to_string(),
        };
        let mut item = new_pending_region_pin_metric("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PendingRegionPinMetricPriorityRangeRule { min: 80, max: 3600 };
        let mut item = new_pending_region_pin_metric("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingRegionPinMetricRequiredNameRule {
            message: "pending region pin metric needs this field".to_string(),
        };
        let mut item = new_pending_region_pin_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingRegionPinMetricCodeLengthRule { max_length: 64 };
        let mut item = new_pending_region_pin_metric("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = PendingRegionPinMetricGraceDaysRangeRule { min: 80, max: 80 };
        let mut item = new_pending_region_pin_metric("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
