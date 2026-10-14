use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering snapshot must pass. Check returns its problems, or none.
pub trait PeeringSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &PeeringSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSnapshotRequiredTitleRule {
    pub message: String,
}

impl PeeringSnapshotRule for PeeringSnapshotRequiredTitleRule {
    fn check(&self, item: &PeeringSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl PeeringSnapshotRule for PeeringSnapshotTitleLengthRule {
    fn check(&self, item: &PeeringSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PeeringSnapshotRule for PeeringSnapshotAllowedCodeRule {
    fn check(&self, item: &PeeringSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering snapshot.
#[derive(Debug, Clone)]
pub struct PeeringSnapshotValidator {
    pub rules: Vec<Rc<dyn PeeringSnapshotRule>>,
}

impl PeeringSnapshotValidator {
    pub fn validate(&self, item: &PeeringSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_snapshot_validator() -> PeeringSnapshotValidator {
    PeeringSnapshotValidator {
        rules: vec![
            Rc::new(PeeringSnapshotRequiredTitleRule {
                message: "peering snapshot needs this field".to_string(),
            }) as Rc<dyn PeeringSnapshotRule>,
            Rc::new(PeeringSnapshotTitleLengthRule { max_length: 255 }),
            Rc::new(PeeringSnapshotAllowedCodeRule {
                allowed: vec!["manual".to_string(), "global".to_string(), "capped".to_string()],
            }),
        ],
    }
}
