use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log quota must pass. Check returns its problems, or none.
pub trait FlowLogQuotaRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogQuotaAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogQuotaRule for FlowLogQuotaAttemptsRangeRule {
    fn check(&self, item: &FlowLogQuota) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogQuotaRequiredRegionRule {
    pub message: String,
}

impl FlowLogQuotaRule for FlowLogQuotaRequiredRegionRule {
    fn check(&self, item: &FlowLogQuota) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogQuotaRequiredLabelRule {
    pub message: String,
}

impl FlowLogQuotaRule for FlowLogQuotaRequiredLabelRule {
    fn check(&self, item: &FlowLogQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogQuotaLabelLengthRule {
    pub max_length: i64,
}

impl FlowLogQuotaRule for FlowLogQuotaLabelLengthRule {
    fn check(&self, item: &FlowLogQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogQuotaRegionLengthRule {
    pub max_length: i64,
}

impl FlowLogQuotaRule for FlowLogQuotaRegionLengthRule {
    fn check(&self, item: &FlowLogQuota) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log quota.
#[derive(Debug, Clone)]
pub struct FlowLogQuotaValidator {
    pub rules: Vec<Rc<dyn FlowLogQuotaRule>>,
}

impl FlowLogQuotaValidator {
    pub fn validate(&self, item: &FlowLogQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_quota_validator() -> FlowLogQuotaValidator {
    FlowLogQuotaValidator {
        rules: vec![
            Rc::new(FlowLogQuotaAttemptsRangeRule { min: 120, max: 3600 }) as Rc<dyn FlowLogQuotaRule>,
            Rc::new(FlowLogQuotaRequiredRegionRule {
                message: "flow log quota needs an entry".to_string(),
            }),
            Rc::new(FlowLogQuotaRequiredLabelRule {
                message: "flow log quota needs an entry".to_string(),
            }),
            Rc::new(FlowLogQuotaLabelLengthRule { max_length: 255 }),
            Rc::new(FlowLogQuotaRegionLengthRule { max_length: 255 }),
        ],
    }
}
