use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default credential score must pass. Check returns its problems, or none.
pub trait DefaultCredentialScoreRule: std::fmt::Debug {
    fn check(&self, item: &DefaultCredentialScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialScoreAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DefaultCredentialScoreRule for DefaultCredentialScoreAllowedTitleRule {
    fn check(&self, item: &DefaultCredentialScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialScoreRequiredCodeRule {
    pub message: String,
}

impl DefaultCredentialScoreRule for DefaultCredentialScoreRequiredCodeRule {
    fn check(&self, item: &DefaultCredentialScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialScoreCodeLengthRule {
    pub max_length: i64,
}

impl DefaultCredentialScoreRule for DefaultCredentialScoreCodeLengthRule {
    fn check(&self, item: &DefaultCredentialScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default credential score.
#[derive(Debug, Clone)]
pub struct DefaultCredentialScoreValidator {
    pub rules: Vec<Rc<dyn DefaultCredentialScoreRule>>,
}

impl DefaultCredentialScoreValidator {
    pub fn validate(&self, item: &DefaultCredentialScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultCredentialScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultCredentialScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_credential_score_validator() -> DefaultCredentialScoreValidator {
    DefaultCredentialScoreValidator {
        rules: vec![
            Rc::new(DefaultCredentialScoreAllowedTitleRule {
                allowed: vec!["fallback".to_string(), "adaptive".to_string()],
            }) as Rc<dyn DefaultCredentialScoreRule>,
            Rc::new(DefaultCredentialScoreRequiredCodeRule {
                message: "default credential score needs a value".to_string(),
            }),
            Rc::new(DefaultCredentialScoreCodeLengthRule { max_length: 80 }),
        ],
    }
}
