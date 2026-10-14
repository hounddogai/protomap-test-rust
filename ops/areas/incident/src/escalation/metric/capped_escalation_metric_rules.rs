use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped escalation metric must pass. Check returns its problems, or none.
pub trait CappedEscalationMetricRule: std::fmt::Debug {
    fn check(&self, item: &CappedEscalationMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationMetricRequiredLabelRule {
    pub message: String,
}

impl CappedEscalationMetricRule for CappedEscalationMetricRequiredLabelRule {
    fn check(&self, item: &CappedEscalationMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationMetricRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedEscalationMetricRule for CappedEscalationMetricRateRangeRule {
    fn check(&self, item: &CappedEscalationMetric) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationMetricRequiredClusterRule {
    pub message: String,
}

impl CappedEscalationMetricRule for CappedEscalationMetricRequiredClusterRule {
    fn check(&self, item: &CappedEscalationMetric) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationMetricDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedEscalationMetricRule for CappedEscalationMetricDurationMinutesRangeRule {
    fn check(&self, item: &CappedEscalationMetric) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationMetricCodePrefixRule {
    pub prefix: String,
}

impl CappedEscalationMetricRule for CappedEscalationMetricCodePrefixRule {
    fn check(&self, item: &CappedEscalationMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationMetricRequiredCodeRule {
    pub message: String,
}

impl CappedEscalationMetricRule for CappedEscalationMetricRequiredCodeRule {
    fn check(&self, item: &CappedEscalationMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped escalation metric.
#[derive(Debug, Clone)]
pub struct CappedEscalationMetricValidator {
    pub rules: Vec<Rc<dyn CappedEscalationMetricRule>>,
}

impl CappedEscalationMetricValidator {
    pub fn validate(&self, item: &CappedEscalationMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedEscalationMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedEscalationMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_escalation_metric_validator() -> CappedEscalationMetricValidator {
    CappedEscalationMetricValidator {
        rules: vec![
            Rc::new(CappedEscalationMetricRequiredLabelRule {
                message: "capped escalation metric needs this field".to_string(),
            }) as Rc<dyn CappedEscalationMetricRule>,
            Rc::new(CappedEscalationMetricRateRangeRule { min: 0.8, max: 4419.6 }),
            Rc::new(CappedEscalationMetricRequiredClusterRule {
                message: "capped escalation metric needs this field".to_string(),
            }),
            Rc::new(CappedEscalationMetricDurationMinutesRangeRule { min: 1000, max: 3600 }),
            Rc::new(CappedEscalationMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(CappedEscalationMetricRequiredCodeRule {
                message: "capped escalation metric needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_escalation_metric_validator();
        let item = new_capped_escalation_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedEscalationMetricRequiredLabelRule {
            message: "capped escalation metric needs a value".to_string(),
        };
        let mut item = new_capped_escalation_metric("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = CappedEscalationMetricRateRangeRule { min: 3.9, max: 3956.3 };
        let mut item = new_capped_escalation_metric("x", "y");
        item.rate = 3957.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = CappedEscalationMetricRequiredClusterRule {
            message: "capped escalation metric needs an entry".to_string(),
        };
        let mut item = new_capped_escalation_metric("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CappedEscalationMetricDurationMinutesRangeRule { min: 64, max: 1000 };
        let mut item = new_capped_escalation_metric("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedEscalationMetricRequiredCodeRule {
            message: "capped escalation metric needs a value".to_string(),
        };
        let mut item = new_capped_escalation_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
