use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard estimate must pass. Check returns its problems, or none.
pub trait DashboardEstimateRule: std::fmt::Debug {
    fn check(&self, item: &DashboardEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardEstimateCodeLengthRule {
    pub max_length: i64,
}

impl DashboardEstimateRule for DashboardEstimateCodeLengthRule {
    fn check(&self, item: &DashboardEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardEstimateCategoryLengthRule {
    pub max_length: i64,
}

impl DashboardEstimateRule for DashboardEstimateCategoryLengthRule {
    fn check(&self, item: &DashboardEstimate) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardEstimateArchivedGuardRule {
    pub limit: f64,
}

impl DashboardEstimateRule for DashboardEstimateArchivedGuardRule {
    fn check(&self, item: &DashboardEstimate) -> Vec<String> {
        if item.archived && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while archived", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardEstimateCodePrefixRule {
    pub prefix: String,
}

impl DashboardEstimateRule for DashboardEstimateCodePrefixRule {
    fn check(&self, item: &DashboardEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardEstimateRequiredCipherRule {
    pub message: String,
}

impl DashboardEstimateRule for DashboardEstimateRequiredCipherRule {
    fn check(&self, item: &DashboardEstimate) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard estimate.
#[derive(Debug, Clone)]
pub struct DashboardEstimateValidator {
    pub rules: Vec<Rc<dyn DashboardEstimateRule>>,
}

impl DashboardEstimateValidator {
    pub fn validate(&self, item: &DashboardEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_estimate_validator() -> DashboardEstimateValidator {
    DashboardEstimateValidator {
        rules: vec![
            Rc::new(DashboardEstimateCodeLengthRule { max_length: 3600 }) as Rc<dyn DashboardEstimateRule>,
            Rc::new(DashboardEstimateCategoryLengthRule { max_length: 16 }),
            Rc::new(DashboardEstimateArchivedGuardRule { limit: 6.4 }),
            Rc::new(DashboardEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DashboardEstimateRequiredCipherRule {
                message: "dashboard estimate needs an entry".to_string(),
            }),
        ],
    }
}
