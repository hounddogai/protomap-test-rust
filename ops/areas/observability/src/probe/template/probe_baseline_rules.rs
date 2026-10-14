use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe baseline must pass. Check returns its problems, or none.
pub trait ProbeBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ProbeBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBaselineAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl ProbeBaselineRule for ProbeBaselineAllowedOwnerIdRule {
    fn check(&self, item: &ProbeBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is blocked", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBaselineCodeLengthRule {
    pub max_length: i64,
}

impl ProbeBaselineRule for ProbeBaselineCodeLengthRule {
    fn check(&self, item: &ProbeBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBaselineRequiredTitleRule {
    pub message: String,
}

impl ProbeBaselineRule for ProbeBaselineRequiredTitleRule {
    fn check(&self, item: &ProbeBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBaselineOwnerIdLengthRule {
    pub max_length: i64,
}

impl ProbeBaselineRule for ProbeBaselineOwnerIdLengthRule {
    fn check(&self, item: &ProbeBaseline) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBaselineRequiredOwnerIdRule {
    pub message: String,
}

impl ProbeBaselineRule for ProbeBaselineRequiredOwnerIdRule {
    fn check(&self, item: &ProbeBaseline) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBaselineCodePrefixRule {
    pub prefix: String,
}

impl ProbeBaselineRule for ProbeBaselineCodePrefixRule {
    fn check(&self, item: &ProbeBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe baseline.
#[derive(Debug, Clone)]
pub struct ProbeBaselineValidator {
    pub rules: Vec<Rc<dyn ProbeBaselineRule>>,
}

impl ProbeBaselineValidator {
    pub fn validate(&self, item: &ProbeBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_baseline_validator() -> ProbeBaselineValidator {
    ProbeBaselineValidator {
        rules: vec![
            Rc::new(ProbeBaselineAllowedOwnerIdRule {
                allowed: vec!["fallback".to_string()],
            }) as Rc<dyn ProbeBaselineRule>,
            Rc::new(ProbeBaselineCodeLengthRule { max_length: 1000 }),
            Rc::new(ProbeBaselineRequiredTitleRule {
                message: "probe baseline needs a value".to_string(),
            }),
            Rc::new(ProbeBaselineOwnerIdLengthRule { max_length: 32 }),
            Rc::new(ProbeBaselineRequiredOwnerIdRule {
                message: "probe baseline needs this field".to_string(),
            }),
            Rc::new(ProbeBaselineCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
