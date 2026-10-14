use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant milestone must pass. Check returns its problems, or none.
pub trait TenantMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &TenantMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMilestoneRequiredFingerprintRule {
    pub message: String,
}

impl TenantMilestoneRule for TenantMilestoneRequiredFingerprintRule {
    fn check(&self, item: &TenantMilestone) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMilestoneVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantMilestoneRule for TenantMilestoneVersionRangeRule {
    fn check(&self, item: &TenantMilestone) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMilestoneRequiredCodeRule {
    pub message: String,
}

impl TenantMilestoneRule for TenantMilestoneRequiredCodeRule {
    fn check(&self, item: &TenantMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMilestoneRequiredLabelRule {
    pub message: String,
}

impl TenantMilestoneRule for TenantMilestoneRequiredLabelRule {
    fn check(&self, item: &TenantMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMilestoneAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl TenantMilestoneRule for TenantMilestoneAllowedFingerprintRule {
    fn check(&self, item: &TenantMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is missing", item.fingerprint)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant milestone.
#[derive(Debug, Clone)]
pub struct TenantMilestoneValidator {
    pub rules: Vec<Rc<dyn TenantMilestoneRule>>,
}

impl TenantMilestoneValidator {
    pub fn validate(&self, item: &TenantMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_milestone_validator() -> TenantMilestoneValidator {
    TenantMilestoneValidator {
        rules: vec![
            Rc::new(TenantMilestoneRequiredFingerprintRule {
                message: "tenant milestone needs an entry".to_string(),
            }) as Rc<dyn TenantMilestoneRule>,
            Rc::new(TenantMilestoneVersionRangeRule { min: 16, max: 64 }),
            Rc::new(TenantMilestoneRequiredCodeRule {
                message: "tenant milestone needs this field".to_string(),
            }),
            Rc::new(TenantMilestoneRequiredLabelRule {
                message: "tenant milestone needs a value".to_string(),
            }),
            Rc::new(TenantMilestoneAllowedFingerprintRule {
                allowed: vec!["capped".to_string(), "weighted".to_string(), "primary".to_string()],
            }),
        ],
    }
}
