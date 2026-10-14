use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending redaction bundle must pass. Check returns its problems, or none.
pub trait PendingRedactionBundleRule: std::fmt::Debug {
    fn check(&self, item: &PendingRedactionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionBundleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRedactionBundleRule for PendingRedactionBundleBurstRangeRule {
    fn check(&self, item: &PendingRedactionBundle) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionBundleAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl PendingRedactionBundleRule for PendingRedactionBundleAllowedClusterRule {
    fn check(&self, item: &PendingRedactionBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is unknown", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionBundleCodePrefixRule {
    pub prefix: String,
}

impl PendingRedactionBundleRule for PendingRedactionBundleCodePrefixRule {
    fn check(&self, item: &PendingRedactionBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionBundleRequiredClusterRule {
    pub message: String,
}

impl PendingRedactionBundleRule for PendingRedactionBundleRequiredClusterRule {
    fn check(&self, item: &PendingRedactionBundle) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionBundleRequiredSourceRule {
    pub message: String,
}

impl PendingRedactionBundleRule for PendingRedactionBundleRequiredSourceRule {
    fn check(&self, item: &PendingRedactionBundle) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionBundleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRedactionBundleRule for PendingRedactionBundleFloorValueRangeRule {
    fn check(&self, item: &PendingRedactionBundle) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending redaction bundle.
#[derive(Debug, Clone)]
pub struct PendingRedactionBundleValidator {
    pub rules: Vec<Rc<dyn PendingRedactionBundleRule>>,
}

impl PendingRedactionBundleValidator {
    pub fn validate(&self, item: &PendingRedactionBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRedactionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRedactionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_redaction_bundle_validator() -> PendingRedactionBundleValidator {
    PendingRedactionBundleValidator {
        rules: vec![
            Rc::new(PendingRedactionBundleBurstRangeRule { min: 16, max: 80 }) as Rc<dyn PendingRedactionBundleRule>,
            Rc::new(PendingRedactionBundleAllowedClusterRule {
                allowed: vec!["fallback".to_string(), "legacy".to_string()],
            }),
            Rc::new(PendingRedactionBundleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PendingRedactionBundleRequiredClusterRule {
                message: "pending redaction bundle needs an entry".to_string(),
            }),
            Rc::new(PendingRedactionBundleRequiredSourceRule {
                message: "pending redaction bundle needs this field".to_string(),
            }),
            Rc::new(PendingRedactionBundleFloorValueRangeRule { min: 2.7, max: 4581.4 }),
        ],
    }
}
