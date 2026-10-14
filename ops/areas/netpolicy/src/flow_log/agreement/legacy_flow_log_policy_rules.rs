use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy flow log policy must pass. Check returns its problems, or none.
pub trait LegacyFlowLogPolicyRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFlowLogPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogPolicyLabelLengthRule {
    pub max_length: i64,
}

impl LegacyFlowLogPolicyRule for LegacyFlowLogPolicyLabelLengthRule {
    fn check(&self, item: &LegacyFlowLogPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogPolicyRequiredClusterRule {
    pub message: String,
}

impl LegacyFlowLogPolicyRule for LegacyFlowLogPolicyRequiredClusterRule {
    fn check(&self, item: &LegacyFlowLogPolicy) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogPolicyAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyFlowLogPolicyRule for LegacyFlowLogPolicyAllowedCodeRule {
    fn check(&self, item: &LegacyFlowLogPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogPolicyBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyFlowLogPolicyRule for LegacyFlowLogPolicyBurstRangeRule {
    fn check(&self, item: &LegacyFlowLogPolicy) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogPolicyExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyFlowLogPolicyRule for LegacyFlowLogPolicyExpiresAtRangeRule {
    fn check(&self, item: &LegacyFlowLogPolicy) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy flow log policy.
#[derive(Debug, Clone)]
pub struct LegacyFlowLogPolicyValidator {
    pub rules: Vec<Rc<dyn LegacyFlowLogPolicyRule>>,
}

impl LegacyFlowLogPolicyValidator {
    pub fn validate(&self, item: &LegacyFlowLogPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyFlowLogPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFlowLogPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_flow_log_policy_validator() -> LegacyFlowLogPolicyValidator {
    LegacyFlowLogPolicyValidator {
        rules: vec![
            Rc::new(LegacyFlowLogPolicyLabelLengthRule { max_length: 1000 }) as Rc<dyn LegacyFlowLogPolicyRule>,
            Rc::new(LegacyFlowLogPolicyRequiredClusterRule {
                message: "legacy flow log policy needs this field".to_string(),
            }),
            Rc::new(LegacyFlowLogPolicyAllowedCodeRule {
                allowed: vec!["global".to_string(), "capped".to_string()],
            }),
            Rc::new(LegacyFlowLogPolicyBurstRangeRule { min: 32, max: 3600 }),
            Rc::new(LegacyFlowLogPolicyExpiresAtRangeRule { min: 80, max: 86400 }),
        ],
    }
}
