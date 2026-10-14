use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual escalation adjustment must pass. Check returns its problems, or none.
pub trait ManualEscalationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &ManualEscalationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEscalationAdjustmentChannelLengthRule {
    pub max_length: i64,
}

impl ManualEscalationAdjustmentRule for ManualEscalationAdjustmentChannelLengthRule {
    fn check(&self, item: &ManualEscalationAdjustment) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEscalationAdjustmentRequiredTitleRule {
    pub message: String,
}

impl ManualEscalationAdjustmentRule for ManualEscalationAdjustmentRequiredTitleRule {
    fn check(&self, item: &ManualEscalationAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEscalationAdjustmentAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl ManualEscalationAdjustmentRule for ManualEscalationAdjustmentAllowedChannelRule {
    fn check(&self, item: &ManualEscalationAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is too long", item.channel)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual escalation adjustment.
#[derive(Debug, Clone)]
pub struct ManualEscalationAdjustmentValidator {
    pub rules: Vec<Rc<dyn ManualEscalationAdjustmentRule>>,
}

impl ManualEscalationAdjustmentValidator {
    pub fn validate(&self, item: &ManualEscalationAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualEscalationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualEscalationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_escalation_adjustment_validator() -> ManualEscalationAdjustmentValidator {
    ManualEscalationAdjustmentValidator {
        rules: vec![
            Rc::new(ManualEscalationAdjustmentChannelLengthRule { max_length: 1000 })
                as Rc<dyn ManualEscalationAdjustmentRule>,
            Rc::new(ManualEscalationAdjustmentRequiredTitleRule {
                message: "manual escalation adjustment needs a value".to_string(),
            }),
            Rc::new(ManualEscalationAdjustmentAllowedChannelRule {
                allowed: vec!["default".to_string()],
            }),
        ],
    }
}
