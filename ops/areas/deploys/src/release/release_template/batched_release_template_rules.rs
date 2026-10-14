use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched release template must pass. Check returns its problems, or none.
pub trait BatchedReleaseTemplateRule: std::fmt::Debug {
    fn check(&self, item: &BatchedReleaseTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseTemplateIssuerLengthRule {
    pub max_length: i64,
}

impl BatchedReleaseTemplateRule for BatchedReleaseTemplateIssuerLengthRule {
    fn check(&self, item: &BatchedReleaseTemplate) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseTemplateArchivedGuardRule {
    pub limit: f64,
}

impl BatchedReleaseTemplateRule for BatchedReleaseTemplateArchivedGuardRule {
    fn check(&self, item: &BatchedReleaseTemplate) -> Vec<String> {
        if item.archived && (item.maximum > self.limit) {
            return vec![format!(
                "maximum {} exceeds {} while archived",
                item.maximum, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseTemplateMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedReleaseTemplateRule for BatchedReleaseTemplateMaximumRangeRule {
    fn check(&self, item: &BatchedReleaseTemplate) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseTemplateTitleLengthRule {
    pub max_length: i64,
}

impl BatchedReleaseTemplateRule for BatchedReleaseTemplateTitleLengthRule {
    fn check(&self, item: &BatchedReleaseTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseTemplateRequiredTitleRule {
    pub message: String,
}

impl BatchedReleaseTemplateRule for BatchedReleaseTemplateRequiredTitleRule {
    fn check(&self, item: &BatchedReleaseTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseTemplateCodePrefixRule {
    pub prefix: String,
}

impl BatchedReleaseTemplateRule for BatchedReleaseTemplateCodePrefixRule {
    fn check(&self, item: &BatchedReleaseTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched release template.
#[derive(Debug, Clone)]
pub struct BatchedReleaseTemplateValidator {
    pub rules: Vec<Rc<dyn BatchedReleaseTemplateRule>>,
}

impl BatchedReleaseTemplateValidator {
    pub fn validate(&self, item: &BatchedReleaseTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedReleaseTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedReleaseTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_release_template_validator() -> BatchedReleaseTemplateValidator {
    BatchedReleaseTemplateValidator {
        rules: vec![
            Rc::new(BatchedReleaseTemplateIssuerLengthRule { max_length: 86400 }) as Rc<dyn BatchedReleaseTemplateRule>,
            Rc::new(BatchedReleaseTemplateArchivedGuardRule { limit: 0.6 }),
            Rc::new(BatchedReleaseTemplateMaximumRangeRule { min: 2.4, max: 4157.5 }),
            Rc::new(BatchedReleaseTemplateTitleLengthRule { max_length: 3600 }),
            Rc::new(BatchedReleaseTemplateRequiredTitleRule {
                message: "batched release template needs an entry".to_string(),
            }),
            Rc::new(BatchedReleaseTemplateCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
