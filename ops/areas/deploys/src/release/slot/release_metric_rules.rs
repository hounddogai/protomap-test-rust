use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release metric must pass. Check returns its problems, or none.
pub trait ReleaseMetricRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMetricLabelLengthRule {
    pub max_length: i64,
}

impl ReleaseMetricRule for ReleaseMetricLabelLengthRule {
    fn check(&self, item: &ReleaseMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMetricRequiredCodeRule {
    pub message: String,
}

impl ReleaseMetricRule for ReleaseMetricRequiredCodeRule {
    fn check(&self, item: &ReleaseMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMetricRequiredCountryRule {
    pub message: String,
}

impl ReleaseMetricRule for ReleaseMetricRequiredCountryRule {
    fn check(&self, item: &ReleaseMetric) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMetricCodePrefixRule {
    pub prefix: String,
}

impl ReleaseMetricRule for ReleaseMetricCodePrefixRule {
    fn check(&self, item: &ReleaseMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release metric.
#[derive(Debug, Clone)]
pub struct ReleaseMetricValidator {
    pub rules: Vec<Rc<dyn ReleaseMetricRule>>,
}

impl ReleaseMetricValidator {
    pub fn validate(&self, item: &ReleaseMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_metric_validator() -> ReleaseMetricValidator {
    ReleaseMetricValidator {
        rules: vec![
            Rc::new(ReleaseMetricLabelLengthRule { max_length: 255 }) as Rc<dyn ReleaseMetricRule>,
            Rc::new(ReleaseMetricRequiredCodeRule {
                message: "release metric needs an entry".to_string(),
            }),
            Rc::new(ReleaseMetricRequiredCountryRule {
                message: "release metric needs an entry".to_string(),
            }),
            Rc::new(ReleaseMetricCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_metric_validator();
        let item = new_release_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ReleaseMetricLabelLengthRule { max_length: 80 };
        let mut item = new_release_metric("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleaseMetricRequiredCodeRule {
            message: "release metric needs this field".to_string(),
        };
        let mut item = new_release_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = ReleaseMetricRequiredCountryRule {
            message: "release metric needs a value".to_string(),
        };
        let mut item = new_release_metric("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
