use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive identity target must pass. Check returns its problems, or none.
pub trait AdaptiveIdentityTargetRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveIdentityTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityTargetCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveIdentityTargetRule for AdaptiveIdentityTargetCodePrefixRule {
    fn check(&self, item: &AdaptiveIdentityTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityTargetRequiredSegmentRule {
    pub message: String,
}

impl AdaptiveIdentityTargetRule for AdaptiveIdentityTargetRequiredSegmentRule {
    fn check(&self, item: &AdaptiveIdentityTarget) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityTargetRequiredTitleRule {
    pub message: String,
}

impl AdaptiveIdentityTargetRule for AdaptiveIdentityTargetRequiredTitleRule {
    fn check(&self, item: &AdaptiveIdentityTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive identity target.
#[derive(Debug, Clone)]
pub struct AdaptiveIdentityTargetValidator {
    pub rules: Vec<Rc<dyn AdaptiveIdentityTargetRule>>,
}

impl AdaptiveIdentityTargetValidator {
    pub fn validate(&self, item: &AdaptiveIdentityTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveIdentityTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveIdentityTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_identity_target_validator() -> AdaptiveIdentityTargetValidator {
    AdaptiveIdentityTargetValidator {
        rules: vec![
            Rc::new(AdaptiveIdentityTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn AdaptiveIdentityTargetRule>,
            Rc::new(AdaptiveIdentityTargetRequiredSegmentRule {
                message: "adaptive identity target needs this field".to_string(),
            }),
            Rc::new(AdaptiveIdentityTargetRequiredTitleRule {
                message: "adaptive identity target needs this field".to_string(),
            }),
        ],
    }
}
