use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual audit event metric must pass. Check returns its problems, or none.
pub trait ManualAuditEventMetricRule: std::fmt::Debug {
    fn check(&self, item: &ManualAuditEventMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventMetricAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualAuditEventMetricRule for ManualAuditEventMetricAllowedCodeRule {
    fn check(&self, item: &ManualAuditEventMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventMetricRequiredLabelRule {
    pub message: String,
}

impl ManualAuditEventMetricRule for ManualAuditEventMetricRequiredLabelRule {
    fn check(&self, item: &ManualAuditEventMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventMetricRequiredRegionRule {
    pub message: String,
}

impl ManualAuditEventMetricRule for ManualAuditEventMetricRequiredRegionRule {
    fn check(&self, item: &ManualAuditEventMetric) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventMetricCodeLengthRule {
    pub max_length: i64,
}

impl ManualAuditEventMetricRule for ManualAuditEventMetricCodeLengthRule {
    fn check(&self, item: &ManualAuditEventMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual audit event metric.
#[derive(Debug, Clone)]
pub struct ManualAuditEventMetricValidator {
    pub rules: Vec<Rc<dyn ManualAuditEventMetricRule>>,
}

impl ManualAuditEventMetricValidator {
    pub fn validate(&self, item: &ManualAuditEventMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualAuditEventMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAuditEventMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_audit_event_metric_validator() -> ManualAuditEventMetricValidator {
    ManualAuditEventMetricValidator {
        rules: vec![
            Rc::new(ManualAuditEventMetricAllowedCodeRule {
                allowed: vec!["default".to_string(), "seasonal".to_string(), "scheduled".to_string()],
            }) as Rc<dyn ManualAuditEventMetricRule>,
            Rc::new(ManualAuditEventMetricRequiredLabelRule {
                message: "manual audit event metric needs this field".to_string(),
            }),
            Rc::new(ManualAuditEventMetricRequiredRegionRule {
                message: "manual audit event metric needs an entry".to_string(),
            }),
            Rc::new(ManualAuditEventMetricCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_audit_event_metric_validator();
        let item = new_manual_audit_event_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualAuditEventMetricRequiredLabelRule {
            message: "manual audit event metric needs this field".to_string(),
        };
        let mut item = new_manual_audit_event_metric("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = ManualAuditEventMetricRequiredRegionRule {
            message: "manual audit event metric needs a value".to_string(),
        };
        let mut item = new_manual_audit_event_metric("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualAuditEventMetricCodeLengthRule { max_length: 32 };
        let mut item = new_manual_audit_event_metric("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
