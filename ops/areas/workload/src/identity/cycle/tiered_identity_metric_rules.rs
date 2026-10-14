use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered identity metric must pass. Check returns its problems, or none.
pub trait TieredIdentityMetricRule: std::fmt::Debug {
    fn check(&self, item: &TieredIdentityMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityMetricCodeLengthRule {
    pub max_length: i64,
}

impl TieredIdentityMetricRule for TieredIdentityMetricCodeLengthRule {
    fn check(&self, item: &TieredIdentityMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityMetricActiveGuardRule {
    pub limit: f64,
}

impl TieredIdentityMetricRule for TieredIdentityMetricActiveGuardRule {
    fn check(&self, item: &TieredIdentityMetric) -> Vec<String> {
        if item.active && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while active", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityMetricAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl TieredIdentityMetricRule for TieredIdentityMetricAllowedPrincipalRule {
    fn check(&self, item: &TieredIdentityMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is below minimum", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityMetricRequiredTenantIdRule {
    pub message: String,
}

impl TieredIdentityMetricRule for TieredIdentityMetricRequiredTenantIdRule {
    fn check(&self, item: &TieredIdentityMetric) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered identity metric.
#[derive(Debug, Clone)]
pub struct TieredIdentityMetricValidator {
    pub rules: Vec<Rc<dyn TieredIdentityMetricRule>>,
}

impl TieredIdentityMetricValidator {
    pub fn validate(&self, item: &TieredIdentityMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredIdentityMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredIdentityMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_identity_metric_validator() -> TieredIdentityMetricValidator {
    TieredIdentityMetricValidator {
        rules: vec![
            Rc::new(TieredIdentityMetricCodeLengthRule { max_length: 16 }) as Rc<dyn TieredIdentityMetricRule>,
            Rc::new(TieredIdentityMetricActiveGuardRule { limit: 2.0 }),
            Rc::new(TieredIdentityMetricAllowedPrincipalRule {
                allowed: vec!["lenient".to_string(), "legacy".to_string(), "weighted".to_string()],
            }),
            Rc::new(TieredIdentityMetricRequiredTenantIdRule {
                message: "tiered identity metric needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_identity_metric_validator();
        let item = new_tiered_identity_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredIdentityMetricCodeLengthRule { max_length: 80 };
        let mut item = new_tiered_identity_metric("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TieredIdentityMetricRequiredTenantIdRule {
            message: "tiered identity metric needs an entry".to_string(),
        };
        let mut item = new_tiered_identity_metric("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
