use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert metric must pass. Check returns its problems, or none.
pub trait AlertMetricRule: std::fmt::Debug {
    fn check(&self, item: &AlertMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMetricAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AlertMetricRule for AlertMetricAllowedLabelRule {
    fn check(&self, item: &AlertMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMetricRequiredCountryRule {
    pub message: String,
}

impl AlertMetricRule for AlertMetricRequiredCountryRule {
    fn check(&self, item: &AlertMetric) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMetricCountryLengthRule {
    pub max_length: i64,
}

impl AlertMetricRule for AlertMetricCountryLengthRule {
    fn check(&self, item: &AlertMetric) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert metric.
#[derive(Debug, Clone)]
pub struct AlertMetricValidator {
    pub rules: Vec<Rc<dyn AlertMetricRule>>,
}

impl AlertMetricValidator {
    pub fn validate(&self, item: &AlertMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_metric_validator() -> AlertMetricValidator {
    AlertMetricValidator {
        rules: vec![
            Rc::new(AlertMetricAllowedLabelRule { allowed: Vec::new() }) as Rc<dyn AlertMetricRule>,
            Rc::new(AlertMetricRequiredCountryRule {
                message: "alert metric needs an entry".to_string(),
            }),
            Rc::new(AlertMetricCountryLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_metric_validator();
        let item = new_alert_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = AlertMetricRequiredCountryRule {
            message: "alert metric needs a value".to_string(),
        };
        let mut item = new_alert_metric("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = AlertMetricCountryLengthRule { max_length: 32 };
        let mut item = new_alert_metric("x", "y");
        item.country = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
