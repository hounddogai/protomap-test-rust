use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped retention metric must pass. Check returns its problems, or none.
pub trait CappedRetentionMetricRule: std::fmt::Debug {
    fn check(&self, item: &CappedRetentionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionMetricAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl CappedRetentionMetricRule for CappedRetentionMetricAllowedTenantIdRule {
    fn check(&self, item: &CappedRetentionMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is inconsistent", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionMetricRegionLengthRule {
    pub max_length: i64,
}

impl CappedRetentionMetricRule for CappedRetentionMetricRegionLengthRule {
    fn check(&self, item: &CappedRetentionMetric) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionMetricCodePrefixRule {
    pub prefix: String,
}

impl CappedRetentionMetricRule for CappedRetentionMetricCodePrefixRule {
    fn check(&self, item: &CappedRetentionMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped retention metric.
#[derive(Debug, Clone)]
pub struct CappedRetentionMetricValidator {
    pub rules: Vec<Rc<dyn CappedRetentionMetricRule>>,
}

impl CappedRetentionMetricValidator {
    pub fn validate(&self, item: &CappedRetentionMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRetentionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRetentionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_retention_metric_validator() -> CappedRetentionMetricValidator {
    CappedRetentionMetricValidator {
        rules: vec![
            Rc::new(CappedRetentionMetricAllowedTenantIdRule { allowed: Vec::new() })
                as Rc<dyn CappedRetentionMetricRule>,
            Rc::new(CappedRetentionMetricRegionLengthRule { max_length: 16 }),
            Rc::new(CappedRetentionMetricCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
