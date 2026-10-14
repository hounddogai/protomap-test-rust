use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation summary must pass. Check returns its problems, or none.
pub trait IsolationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &IsolationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSummaryRequiredClusterRule {
    pub message: String,
}

impl IsolationSummaryRule for IsolationSummaryRequiredClusterRule {
    fn check(&self, item: &IsolationSummary) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSummaryExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationSummaryRule for IsolationSummaryExpiresAtRangeRule {
    fn check(&self, item: &IsolationSummary) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSummaryPinnedGuardRule {
    pub limit: i64,
}

impl IsolationSummaryRule for IsolationSummaryPinnedGuardRule {
    fn check(&self, item: &IsolationSummary) -> Vec<String> {
        if item.pinned && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while pinned",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSummaryRequiredCodeRule {
    pub message: String,
}

impl IsolationSummaryRule for IsolationSummaryRequiredCodeRule {
    fn check(&self, item: &IsolationSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSummaryRequiredLabelRule {
    pub message: String,
}

impl IsolationSummaryRule for IsolationSummaryRequiredLabelRule {
    fn check(&self, item: &IsolationSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSummaryCodePrefixRule {
    pub prefix: String,
}

impl IsolationSummaryRule for IsolationSummaryCodePrefixRule {
    fn check(&self, item: &IsolationSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation summary.
#[derive(Debug, Clone)]
pub struct IsolationSummaryValidator {
    pub rules: Vec<Rc<dyn IsolationSummaryRule>>,
}

impl IsolationSummaryValidator {
    pub fn validate(&self, item: &IsolationSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_summary_validator() -> IsolationSummaryValidator {
    IsolationSummaryValidator {
        rules: vec![
            Rc::new(IsolationSummaryRequiredClusterRule {
                message: "isolation summary needs an entry".to_string(),
            }) as Rc<dyn IsolationSummaryRule>,
            Rc::new(IsolationSummaryExpiresAtRangeRule { min: 80, max: 120 }),
            Rc::new(IsolationSummaryPinnedGuardRule { limit: 32 }),
            Rc::new(IsolationSummaryRequiredCodeRule {
                message: "isolation summary needs an entry".to_string(),
            }),
            Rc::new(IsolationSummaryRequiredLabelRule {
                message: "isolation summary needs a value".to_string(),
            }),
            Rc::new(IsolationSummaryCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
