use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive usage agreement must pass. Check returns its problems, or none.
pub trait AdaptiveUsageAgreementRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveUsageAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageAgreementCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveUsageAgreementRule for AdaptiveUsageAgreementCodeLengthRule {
    fn check(&self, item: &AdaptiveUsageAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageAgreementHostLengthRule {
    pub max_length: i64,
}

impl AdaptiveUsageAgreementRule for AdaptiveUsageAgreementHostLengthRule {
    fn check(&self, item: &AdaptiveUsageAgreement) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageAgreementRequiredCodeRule {
    pub message: String,
}

impl AdaptiveUsageAgreementRule for AdaptiveUsageAgreementRequiredCodeRule {
    fn check(&self, item: &AdaptiveUsageAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive usage agreement.
#[derive(Debug, Clone)]
pub struct AdaptiveUsageAgreementValidator {
    pub rules: Vec<Rc<dyn AdaptiveUsageAgreementRule>>,
}

impl AdaptiveUsageAgreementValidator {
    pub fn validate(&self, item: &AdaptiveUsageAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveUsageAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveUsageAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_usage_agreement_validator() -> AdaptiveUsageAgreementValidator {
    AdaptiveUsageAgreementValidator {
        rules: vec![
            Rc::new(AdaptiveUsageAgreementCodeLengthRule { max_length: 16 }) as Rc<dyn AdaptiveUsageAgreementRule>,
            Rc::new(AdaptiveUsageAgreementHostLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveUsageAgreementRequiredCodeRule {
                message: "adaptive usage agreement needs an entry".to_string(),
            }),
        ],
    }
}
