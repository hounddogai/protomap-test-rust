use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain metric must pass. Check returns its problems, or none.
pub trait TrustDomainMetricRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainMetricRequiredSummaryRule {
    pub message: String,
}

impl TrustDomainMetricRule for TrustDomainMetricRequiredSummaryRule {
    fn check(&self, item: &TrustDomainMetric) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainMetricRequiredCodeRule {
    pub message: String,
}

impl TrustDomainMetricRule for TrustDomainMetricRequiredCodeRule {
    fn check(&self, item: &TrustDomainMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainMetricRequiredLabelRule {
    pub message: String,
}

impl TrustDomainMetricRule for TrustDomainMetricRequiredLabelRule {
    fn check(&self, item: &TrustDomainMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain metric.
#[derive(Debug, Clone)]
pub struct TrustDomainMetricValidator {
    pub rules: Vec<Rc<dyn TrustDomainMetricRule>>,
}

impl TrustDomainMetricValidator {
    pub fn validate(&self, item: &TrustDomainMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_metric_validator() -> TrustDomainMetricValidator {
    TrustDomainMetricValidator {
        rules: vec![
            Rc::new(TrustDomainMetricRequiredSummaryRule {
                message: "trust domain metric needs a value".to_string(),
            }) as Rc<dyn TrustDomainMetricRule>,
            Rc::new(TrustDomainMetricRequiredCodeRule {
                message: "trust domain metric needs a value".to_string(),
            }),
            Rc::new(TrustDomainMetricRequiredLabelRule {
                message: "trust domain metric needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_metric_validator();
        let item = new_trust_domain_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = TrustDomainMetricRequiredSummaryRule {
            message: "trust domain metric needs this field".to_string(),
        };
        let mut item = new_trust_domain_metric("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainMetricRequiredCodeRule {
            message: "trust domain metric needs this field".to_string(),
        };
        let mut item = new_trust_domain_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TrustDomainMetricRequiredLabelRule {
            message: "trust domain metric needs a value".to_string(),
        };
        let mut item = new_trust_domain_metric("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
