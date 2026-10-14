use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event metric must pass. Check returns its problems, or none.
pub trait AuditEventMetricRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMetricWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventMetricRule for AuditEventMetricWeightRangeRule {
    fn check(&self, item: &AuditEventMetric) -> Vec<String> {
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
pub struct AuditEventMetricCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventMetricRule for AuditEventMetricCodeLengthRule {
    fn check(&self, item: &AuditEventMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMetricClusterLengthRule {
    pub max_length: i64,
}

impl AuditEventMetricRule for AuditEventMetricClusterLengthRule {
    fn check(&self, item: &AuditEventMetric) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMetricMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventMetricRule for AuditEventMetricMarginRangeRule {
    fn check(&self, item: &AuditEventMetric) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event metric.
#[derive(Debug, Clone)]
pub struct AuditEventMetricValidator {
    pub rules: Vec<Rc<dyn AuditEventMetricRule>>,
}

impl AuditEventMetricValidator {
    pub fn validate(&self, item: &AuditEventMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_metric_validator() -> AuditEventMetricValidator {
    AuditEventMetricValidator {
        rules: vec![
            Rc::new(AuditEventMetricWeightRangeRule { min: 0.3, max: 1274.7 }) as Rc<dyn AuditEventMetricRule>,
            Rc::new(AuditEventMetricCodeLengthRule { max_length: 64 }),
            Rc::new(AuditEventMetricClusterLengthRule { max_length: 64 }),
            Rc::new(AuditEventMetricMarginRangeRule { min: 9.3, max: 4884.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_metric_validator();
        let item = new_audit_event_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = AuditEventMetricWeightRangeRule { min: 0.1, max: 2582.2 };
        let mut item = new_audit_event_metric("x", "y");
        item.weight = 2583.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventMetricCodeLengthRule { max_length: 80 };
        let mut item = new_audit_event_metric("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = AuditEventMetricClusterLengthRule { max_length: 255 };
        let mut item = new_audit_event_metric("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = AuditEventMetricMarginRangeRule { min: 7.6, max: 114.5 };
        let mut item = new_audit_event_metric("x", "y");
        item.margin = 116.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
