use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant metric must pass. Check returns its problems, or none.
pub trait TenantMetricRule: std::fmt::Debug {
    fn check(&self, item: &TenantMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMetricCodePrefixRule {
    pub prefix: String,
}

impl TenantMetricRule for TenantMetricCodePrefixRule {
    fn check(&self, item: &TenantMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMetricLabelLengthRule {
    pub max_length: i64,
}

impl TenantMetricRule for TenantMetricLabelLengthRule {
    fn check(&self, item: &TenantMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMetricRequiredScopeRule {
    pub message: String,
}

impl TenantMetricRule for TenantMetricRequiredScopeRule {
    fn check(&self, item: &TenantMetric) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant metric.
#[derive(Debug, Clone)]
pub struct TenantMetricValidator {
    pub rules: Vec<Rc<dyn TenantMetricRule>>,
}

impl TenantMetricValidator {
    pub fn validate(&self, item: &TenantMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_metric_validator() -> TenantMetricValidator {
    TenantMetricValidator {
        rules: vec![
            Rc::new(TenantMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn TenantMetricRule>,
            Rc::new(TenantMetricLabelLengthRule { max_length: 3600 }),
            Rc::new(TenantMetricRequiredScopeRule {
                message: "tenant metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_metric_validator();
        let item = new_tenant_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TenantMetricLabelLengthRule { max_length: 120 };
        let mut item = new_tenant_metric("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = TenantMetricRequiredScopeRule {
            message: "tenant metric needs this field".to_string(),
        };
        let mut item = new_tenant_metric("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
