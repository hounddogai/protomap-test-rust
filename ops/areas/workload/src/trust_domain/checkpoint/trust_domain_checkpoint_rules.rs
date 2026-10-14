use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain checkpoint must pass. Check returns its problems, or none.
pub trait TrustDomainCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCheckpointClusterLengthRule {
    pub max_length: i64,
}

impl TrustDomainCheckpointRule for TrustDomainCheckpointClusterLengthRule {
    fn check(&self, item: &TrustDomainCheckpoint) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCheckpointRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainCheckpointRule for TrustDomainCheckpointRateRangeRule {
    fn check(&self, item: &TrustDomainCheckpoint) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCheckpointRequiredCodeRule {
    pub message: String,
}

impl TrustDomainCheckpointRule for TrustDomainCheckpointRequiredCodeRule {
    fn check(&self, item: &TrustDomainCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCheckpointRequiredClusterRule {
    pub message: String,
}

impl TrustDomainCheckpointRule for TrustDomainCheckpointRequiredClusterRule {
    fn check(&self, item: &TrustDomainCheckpoint) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCheckpointAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl TrustDomainCheckpointRule for TrustDomainCheckpointAllowedClusterRule {
    fn check(&self, item: &TrustDomainCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is blocked", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCheckpointNameLengthRule {
    pub max_length: i64,
}

impl TrustDomainCheckpointRule for TrustDomainCheckpointNameLengthRule {
    fn check(&self, item: &TrustDomainCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain checkpoint.
#[derive(Debug, Clone)]
pub struct TrustDomainCheckpointValidator {
    pub rules: Vec<Rc<dyn TrustDomainCheckpointRule>>,
}

impl TrustDomainCheckpointValidator {
    pub fn validate(&self, item: &TrustDomainCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_checkpoint_validator() -> TrustDomainCheckpointValidator {
    TrustDomainCheckpointValidator {
        rules: vec![
            Rc::new(TrustDomainCheckpointClusterLengthRule { max_length: 86400 }) as Rc<dyn TrustDomainCheckpointRule>,
            Rc::new(TrustDomainCheckpointRateRangeRule { min: 6.4, max: 3463.4 }),
            Rc::new(TrustDomainCheckpointRequiredCodeRule {
                message: "trust domain checkpoint needs this field".to_string(),
            }),
            Rc::new(TrustDomainCheckpointRequiredClusterRule {
                message: "trust domain checkpoint needs this field".to_string(),
            }),
            Rc::new(TrustDomainCheckpointAllowedClusterRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(TrustDomainCheckpointNameLengthRule { max_length: 255 }),
        ],
    }
}
