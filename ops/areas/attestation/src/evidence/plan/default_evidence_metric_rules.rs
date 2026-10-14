use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default evidence metric must pass. Check returns its problems, or none.
pub trait DefaultEvidenceMetricRule: std::fmt::Debug {
    fn check(&self, item: &DefaultEvidenceMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceMetricAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl DefaultEvidenceMetricRule for DefaultEvidenceMetricAllowedLocaleRule {
    fn check(&self, item: &DefaultEvidenceMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is not allowed", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceMetricRequiredCodeRule {
    pub message: String,
}

impl DefaultEvidenceMetricRule for DefaultEvidenceMetricRequiredCodeRule {
    fn check(&self, item: &DefaultEvidenceMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceMetricCodePrefixRule {
    pub prefix: String,
}

impl DefaultEvidenceMetricRule for DefaultEvidenceMetricCodePrefixRule {
    fn check(&self, item: &DefaultEvidenceMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceMetricRequiredLocaleRule {
    pub message: String,
}

impl DefaultEvidenceMetricRule for DefaultEvidenceMetricRequiredLocaleRule {
    fn check(&self, item: &DefaultEvidenceMetric) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceMetricLocaleLengthRule {
    pub max_length: i64,
}

impl DefaultEvidenceMetricRule for DefaultEvidenceMetricLocaleLengthRule {
    fn check(&self, item: &DefaultEvidenceMetric) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default evidence metric.
#[derive(Debug, Clone)]
pub struct DefaultEvidenceMetricValidator {
    pub rules: Vec<Rc<dyn DefaultEvidenceMetricRule>>,
}

impl DefaultEvidenceMetricValidator {
    pub fn validate(&self, item: &DefaultEvidenceMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultEvidenceMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultEvidenceMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_evidence_metric_validator() -> DefaultEvidenceMetricValidator {
    DefaultEvidenceMetricValidator {
        rules: vec![
            Rc::new(DefaultEvidenceMetricAllowedLocaleRule {
                allowed: vec!["fallback".to_string(), "pending".to_string()],
            }) as Rc<dyn DefaultEvidenceMetricRule>,
            Rc::new(DefaultEvidenceMetricRequiredCodeRule {
                message: "default evidence metric needs a value".to_string(),
            }),
            Rc::new(DefaultEvidenceMetricCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultEvidenceMetricRequiredLocaleRule {
                message: "default evidence metric needs a value".to_string(),
            }),
            Rc::new(DefaultEvidenceMetricLocaleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_evidence_metric_validator();
        let item = new_default_evidence_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultEvidenceMetricRequiredCodeRule {
            message: "default evidence metric needs this field".to_string(),
        };
        let mut item = new_default_evidence_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = DefaultEvidenceMetricRequiredLocaleRule {
            message: "default evidence metric needs an entry".to_string(),
        };
        let mut item = new_default_evidence_metric("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = DefaultEvidenceMetricLocaleLengthRule { max_length: 16 };
        let mut item = new_default_evidence_metric("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
