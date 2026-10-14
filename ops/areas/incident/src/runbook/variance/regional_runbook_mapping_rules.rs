use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional runbook mapping must pass. Check returns its problems, or none.
pub trait RegionalRunbookMappingRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRunbookMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookMappingBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalRunbookMappingRule for RegionalRunbookMappingBudgetRangeRule {
    fn check(&self, item: &RegionalRunbookMapping) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookMappingCodePrefixRule {
    pub prefix: String,
}

impl RegionalRunbookMappingRule for RegionalRunbookMappingCodePrefixRule {
    fn check(&self, item: &RegionalRunbookMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookMappingAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl RegionalRunbookMappingRule for RegionalRunbookMappingAllowedKeyIdRule {
    fn check(&self, item: &RegionalRunbookMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is stale", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional runbook mapping.
#[derive(Debug, Clone)]
pub struct RegionalRunbookMappingValidator {
    pub rules: Vec<Rc<dyn RegionalRunbookMappingRule>>,
}

impl RegionalRunbookMappingValidator {
    pub fn validate(&self, item: &RegionalRunbookMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalRunbookMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRunbookMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_runbook_mapping_validator() -> RegionalRunbookMappingValidator {
    RegionalRunbookMappingValidator {
        rules: vec![
            Rc::new(RegionalRunbookMappingBudgetRangeRule { min: 8.1, max: 3502.8 })
                as Rc<dyn RegionalRunbookMappingRule>,
            Rc::new(RegionalRunbookMappingCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionalRunbookMappingAllowedKeyIdRule {
                allowed: vec!["legacy".to_string(), "manual".to_string(), "rolling".to_string()],
            }),
        ],
    }
}
