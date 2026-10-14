use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling escalation segment must pass. Check returns its problems, or none.
pub trait RollingEscalationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RollingEscalationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationSegmentAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl RollingEscalationSegmentRule for RollingEscalationSegmentAllowedChecksumRule {
    fn check(&self, item: &RollingEscalationSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is below minimum", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationSegmentRequiredTitleRule {
    pub message: String,
}

impl RollingEscalationSegmentRule for RollingEscalationSegmentRequiredTitleRule {
    fn check(&self, item: &RollingEscalationSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationSegmentCodePrefixRule {
    pub prefix: String,
}

impl RollingEscalationSegmentRule for RollingEscalationSegmentCodePrefixRule {
    fn check(&self, item: &RollingEscalationSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling escalation segment.
#[derive(Debug, Clone)]
pub struct RollingEscalationSegmentValidator {
    pub rules: Vec<Rc<dyn RollingEscalationSegmentRule>>,
}

impl RollingEscalationSegmentValidator {
    pub fn validate(&self, item: &RollingEscalationSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingEscalationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEscalationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_escalation_segment_validator() -> RollingEscalationSegmentValidator {
    RollingEscalationSegmentValidator {
        rules: vec![
            Rc::new(RollingEscalationSegmentAllowedChecksumRule {
                allowed: vec!["lenient".to_string(), "global".to_string()],
            }) as Rc<dyn RollingEscalationSegmentRule>,
            Rc::new(RollingEscalationSegmentRequiredTitleRule {
                message: "rolling escalation segment needs this field".to_string(),
            }),
            Rc::new(RollingEscalationSegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
