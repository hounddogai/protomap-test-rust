use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending health check rule must pass. Check returns its problems, or none.
pub trait PendingHealthCheckRuleRule: std::fmt::Debug {
    fn check(&self, item: &PendingHealthCheckRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl PendingHealthCheckRuleRule for PendingHealthCheckRuleAllowedSegmentRule {
    fn check(&self, item: &PendingHealthCheckRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is too long", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingHealthCheckRuleRule for PendingHealthCheckRuleSequenceRangeRule {
    fn check(&self, item: &PendingHealthCheckRule) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleRequiredSummaryRule {
    pub message: String,
}

impl PendingHealthCheckRuleRule for PendingHealthCheckRuleRequiredSummaryRule {
    fn check(&self, item: &PendingHealthCheckRule) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleCodePrefixRule {
    pub prefix: String,
}

impl PendingHealthCheckRuleRule for PendingHealthCheckRuleCodePrefixRule {
    fn check(&self, item: &PendingHealthCheckRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleSummaryLengthRule {
    pub max_length: i64,
}

impl PendingHealthCheckRuleRule for PendingHealthCheckRuleSummaryLengthRule {
    fn check(&self, item: &PendingHealthCheckRule) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleLabelLengthRule {
    pub max_length: i64,
}

impl PendingHealthCheckRuleRule for PendingHealthCheckRuleLabelLengthRule {
    fn check(&self, item: &PendingHealthCheckRule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending health check rule.
#[derive(Debug, Clone)]
pub struct PendingHealthCheckRuleValidator {
    pub rules: Vec<Rc<dyn PendingHealthCheckRuleRule>>,
}

impl PendingHealthCheckRuleValidator {
    pub fn validate(&self, item: &PendingHealthCheckRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingHealthCheckRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingHealthCheckRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_health_check_rule_validator() -> PendingHealthCheckRuleValidator {
    PendingHealthCheckRuleValidator {
        rules: vec![
            Rc::new(PendingHealthCheckRuleAllowedSegmentRule {
                allowed: vec!["capped".to_string(), "strict".to_string()],
            }) as Rc<dyn PendingHealthCheckRuleRule>,
            Rc::new(PendingHealthCheckRuleSequenceRangeRule { min: 16, max: 1000 }),
            Rc::new(PendingHealthCheckRuleRequiredSummaryRule {
                message: "pending health check rule needs a value".to_string(),
            }),
            Rc::new(PendingHealthCheckRuleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingHealthCheckRuleSummaryLengthRule { max_length: 255 }),
            Rc::new(PendingHealthCheckRuleLabelLengthRule { max_length: 1000 }),
        ],
    }
}
