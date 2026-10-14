use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global plan quota must pass. Check returns its problems, or none.
pub trait GlobalPlanQuotaRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPlanQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanQuotaRequiredPrincipalRule {
    pub message: String,
}

impl GlobalPlanQuotaRule for GlobalPlanQuotaRequiredPrincipalRule {
    fn check(&self, item: &GlobalPlanQuota) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanQuotaAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl GlobalPlanQuotaRule for GlobalPlanQuotaAllowedPrincipalRule {
    fn check(&self, item: &GlobalPlanQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is too long", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanQuotaRequiredNameRule {
    pub message: String,
}

impl GlobalPlanQuotaRule for GlobalPlanQuotaRequiredNameRule {
    fn check(&self, item: &GlobalPlanQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanQuotaTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl GlobalPlanQuotaRule for GlobalPlanQuotaTagsLimitRule {
    fn check(&self, item: &GlobalPlanQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanQuotaNameLengthRule {
    pub max_length: i64,
}

impl GlobalPlanQuotaRule for GlobalPlanQuotaNameLengthRule {
    fn check(&self, item: &GlobalPlanQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global plan quota.
#[derive(Debug, Clone)]
pub struct GlobalPlanQuotaValidator {
    pub rules: Vec<Rc<dyn GlobalPlanQuotaRule>>,
}

impl GlobalPlanQuotaValidator {
    pub fn validate(&self, item: &GlobalPlanQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPlanQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPlanQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_plan_quota_validator() -> GlobalPlanQuotaValidator {
    GlobalPlanQuotaValidator {
        rules: vec![
            Rc::new(GlobalPlanQuotaRequiredPrincipalRule {
                message: "global plan quota needs a value".to_string(),
            }) as Rc<dyn GlobalPlanQuotaRule>,
            Rc::new(GlobalPlanQuotaAllowedPrincipalRule {
                allowed: vec!["fallback".to_string(), "seasonal".to_string(), "adaptive".to_string()],
            }),
            Rc::new(GlobalPlanQuotaRequiredNameRule {
                message: "global plan quota needs an entry".to_string(),
            }),
            Rc::new(GlobalPlanQuotaTagsLimitRule {
                max_count: 64,
                banned: vec!["seasonal".to_string(), "pending".to_string()],
            }),
            Rc::new(GlobalPlanQuotaNameLengthRule { max_length: 1000 }),
        ],
    }
}
