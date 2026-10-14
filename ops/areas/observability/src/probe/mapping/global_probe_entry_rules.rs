use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global probe entry must pass. Check returns its problems, or none.
pub trait GlobalProbeEntryRule: std::fmt::Debug {
    fn check(&self, item: &GlobalProbeEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeEntryAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl GlobalProbeEntryRule for GlobalProbeEntryAllowedFingerprintRule {
    fn check(&self, item: &GlobalProbeEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is not allowed", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeEntryUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalProbeEntryRule for GlobalProbeEntryUpdatedAtRangeRule {
    fn check(&self, item: &GlobalProbeEntry) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeEntryRequiredLabelRule {
    pub message: String,
}

impl GlobalProbeEntryRule for GlobalProbeEntryRequiredLabelRule {
    fn check(&self, item: &GlobalProbeEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeEntryThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalProbeEntryRule for GlobalProbeEntryThresholdRangeRule {
    fn check(&self, item: &GlobalProbeEntry) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global probe entry.
#[derive(Debug, Clone)]
pub struct GlobalProbeEntryValidator {
    pub rules: Vec<Rc<dyn GlobalProbeEntryRule>>,
}

impl GlobalProbeEntryValidator {
    pub fn validate(&self, item: &GlobalProbeEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalProbeEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalProbeEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_probe_entry_validator() -> GlobalProbeEntryValidator {
    GlobalProbeEntryValidator {
        rules: vec![
            Rc::new(GlobalProbeEntryAllowedFingerprintRule {
                allowed: vec!["seasonal".to_string(), "global".to_string(), "rolling".to_string()],
            }) as Rc<dyn GlobalProbeEntryRule>,
            Rc::new(GlobalProbeEntryUpdatedAtRangeRule { min: 16, max: 120 }),
            Rc::new(GlobalProbeEntryRequiredLabelRule {
                message: "global probe entry needs a value".to_string(),
            }),
            Rc::new(GlobalProbeEntryThresholdRangeRule { min: 5.6, max: 1628.7 }),
        ],
    }
}
