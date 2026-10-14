use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence variance must pass. Check returns its problems, or none.
pub trait SilenceVarianceRule: std::fmt::Debug {
    fn check(&self, item: &SilenceVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceVarianceTenantIdLengthRule {
    pub max_length: i64,
}

impl SilenceVarianceRule for SilenceVarianceTenantIdLengthRule {
    fn check(&self, item: &SilenceVariance) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceVarianceClusterLengthRule {
    pub max_length: i64,
}

impl SilenceVarianceRule for SilenceVarianceClusterLengthRule {
    fn check(&self, item: &SilenceVariance) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceVarianceRequiredTenantIdRule {
    pub message: String,
}

impl SilenceVarianceRule for SilenceVarianceRequiredTenantIdRule {
    fn check(&self, item: &SilenceVariance) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence variance.
#[derive(Debug, Clone)]
pub struct SilenceVarianceValidator {
    pub rules: Vec<Rc<dyn SilenceVarianceRule>>,
}

impl SilenceVarianceValidator {
    pub fn validate(&self, item: &SilenceVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_variance_validator() -> SilenceVarianceValidator {
    SilenceVarianceValidator {
        rules: vec![
            Rc::new(SilenceVarianceTenantIdLengthRule { max_length: 86400 }) as Rc<dyn SilenceVarianceRule>,
            Rc::new(SilenceVarianceClusterLengthRule { max_length: 86400 }),
            Rc::new(SilenceVarianceRequiredTenantIdRule {
                message: "silence variance needs a value".to_string(),
            }),
        ],
    }
}
