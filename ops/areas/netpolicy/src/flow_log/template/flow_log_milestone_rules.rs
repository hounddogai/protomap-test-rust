use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log milestone must pass. Check returns its problems, or none.
pub trait FlowLogMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMilestoneRequiredFingerprintRule {
    pub message: String,
}

impl FlowLogMilestoneRule for FlowLogMilestoneRequiredFingerprintRule {
    fn check(&self, item: &FlowLogMilestone) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMilestoneLocaleLengthRule {
    pub max_length: i64,
}

impl FlowLogMilestoneRule for FlowLogMilestoneLocaleLengthRule {
    fn check(&self, item: &FlowLogMilestone) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMilestoneRequiredLocaleRule {
    pub message: String,
}

impl FlowLogMilestoneRule for FlowLogMilestoneRequiredLocaleRule {
    fn check(&self, item: &FlowLogMilestone) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMilestoneAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl FlowLogMilestoneRule for FlowLogMilestoneAllowedIssuerRule {
    fn check(&self, item: &FlowLogMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is not allowed", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl FlowLogMilestoneRule for FlowLogMilestoneTitleLengthRule {
    fn check(&self, item: &FlowLogMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMilestoneRequiredCodeRule {
    pub message: String,
}

impl FlowLogMilestoneRule for FlowLogMilestoneRequiredCodeRule {
    fn check(&self, item: &FlowLogMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log milestone.
#[derive(Debug, Clone)]
pub struct FlowLogMilestoneValidator {
    pub rules: Vec<Rc<dyn FlowLogMilestoneRule>>,
}

impl FlowLogMilestoneValidator {
    pub fn validate(&self, item: &FlowLogMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_milestone_validator() -> FlowLogMilestoneValidator {
    FlowLogMilestoneValidator {
        rules: vec![
            Rc::new(FlowLogMilestoneRequiredFingerprintRule {
                message: "flow log milestone needs this field".to_string(),
            }) as Rc<dyn FlowLogMilestoneRule>,
            Rc::new(FlowLogMilestoneLocaleLengthRule { max_length: 1000 }),
            Rc::new(FlowLogMilestoneRequiredLocaleRule {
                message: "flow log milestone needs an entry".to_string(),
            }),
            Rc::new(FlowLogMilestoneAllowedIssuerRule {
                allowed: vec!["batched".to_string(), "tiered".to_string(), "manual".to_string()],
            }),
            Rc::new(FlowLogMilestoneTitleLengthRule { max_length: 255 }),
            Rc::new(FlowLogMilestoneRequiredCodeRule {
                message: "flow log milestone needs an entry".to_string(),
            }),
        ],
    }
}
