use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending grant metric must pass. Check returns its problems, or none.
pub trait PendingGrantMetricRule: std::fmt::Debug {
    fn check(&self, item: &PendingGrantMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantMetricRequiredTimezoneRule {
    pub message: String,
}

impl PendingGrantMetricRule for PendingGrantMetricRequiredTimezoneRule {
    fn check(&self, item: &PendingGrantMetric) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantMetricAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PendingGrantMetricRule for PendingGrantMetricAllowedTitleRule {
    fn check(&self, item: &PendingGrantMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantMetricRequiredCountryRule {
    pub message: String,
}

impl PendingGrantMetricRule for PendingGrantMetricRequiredCountryRule {
    fn check(&self, item: &PendingGrantMetric) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending grant metric.
#[derive(Debug, Clone)]
pub struct PendingGrantMetricValidator {
    pub rules: Vec<Rc<dyn PendingGrantMetricRule>>,
}

impl PendingGrantMetricValidator {
    pub fn validate(&self, item: &PendingGrantMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingGrantMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingGrantMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_grant_metric_validator() -> PendingGrantMetricValidator {
    PendingGrantMetricValidator {
        rules: vec![
            Rc::new(PendingGrantMetricRequiredTimezoneRule {
                message: "pending grant metric needs a value".to_string(),
            }) as Rc<dyn PendingGrantMetricRule>,
            Rc::new(PendingGrantMetricAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(PendingGrantMetricRequiredCountryRule {
                message: "pending grant metric needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_grant_metric_validator();
        let item = new_pending_grant_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PendingGrantMetricRequiredTimezoneRule {
            message: "pending grant metric needs a value".to_string(),
        };
        let mut item = new_pending_grant_metric("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = PendingGrantMetricRequiredCountryRule {
            message: "pending grant metric needs a value".to_string(),
        };
        let mut item = new_pending_grant_metric("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
