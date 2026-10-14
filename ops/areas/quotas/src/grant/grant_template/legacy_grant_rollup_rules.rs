use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy grant rollup must pass. Check returns its problems, or none.
pub trait LegacyGrantRollupRule: std::fmt::Debug {
    fn check(&self, item: &LegacyGrantRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantRollupRequiredAudienceRule {
    pub message: String,
}

impl LegacyGrantRollupRule for LegacyGrantRollupRequiredAudienceRule {
    fn check(&self, item: &LegacyGrantRollup) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantRollupCodeLengthRule {
    pub max_length: i64,
}

impl LegacyGrantRollupRule for LegacyGrantRollupCodeLengthRule {
    fn check(&self, item: &LegacyGrantRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantRollupAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl LegacyGrantRollupRule for LegacyGrantRollupAllowedReviewerRule {
    fn check(&self, item: &LegacyGrantRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is duplicated", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy grant rollup.
#[derive(Debug, Clone)]
pub struct LegacyGrantRollupValidator {
    pub rules: Vec<Rc<dyn LegacyGrantRollupRule>>,
}

impl LegacyGrantRollupValidator {
    pub fn validate(&self, item: &LegacyGrantRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyGrantRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyGrantRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_grant_rollup_validator() -> LegacyGrantRollupValidator {
    LegacyGrantRollupValidator {
        rules: vec![
            Rc::new(LegacyGrantRollupRequiredAudienceRule {
                message: "legacy grant rollup needs a value".to_string(),
            }) as Rc<dyn LegacyGrantRollupRule>,
            Rc::new(LegacyGrantRollupCodeLengthRule { max_length: 3600 }),
            Rc::new(LegacyGrantRollupAllowedReviewerRule {
                allowed: vec!["adaptive".to_string(), "default".to_string(), "default".to_string()],
            }),
        ],
    }
}
