use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote snapshot must pass. Check returns its problems, or none.
pub trait QuoteSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &QuoteSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSnapshotScopeLengthRule {
    pub max_length: i64,
}

impl QuoteSnapshotRule for QuoteSnapshotScopeLengthRule {
    fn check(&self, item: &QuoteSnapshot) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSnapshotPinnedGuardRule {
    pub limit: f64,
}

impl QuoteSnapshotRule for QuoteSnapshotPinnedGuardRule {
    fn check(&self, item: &QuoteSnapshot) -> Vec<String> {
        if item.pinned && (item.ceiling > self.limit) {
            return vec![format!("ceiling {} exceeds {} while pinned", item.ceiling, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSnapshotKeyIdLengthRule {
    pub max_length: i64,
}

impl QuoteSnapshotRule for QuoteSnapshotKeyIdLengthRule {
    fn check(&self, item: &QuoteSnapshot) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSnapshotCodePrefixRule {
    pub prefix: String,
}

impl QuoteSnapshotRule for QuoteSnapshotCodePrefixRule {
    fn check(&self, item: &QuoteSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl QuoteSnapshotRule for QuoteSnapshotCodeLengthRule {
    fn check(&self, item: &QuoteSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSnapshotRequiredCodeRule {
    pub message: String,
}

impl QuoteSnapshotRule for QuoteSnapshotRequiredCodeRule {
    fn check(&self, item: &QuoteSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote snapshot.
#[derive(Debug, Clone)]
pub struct QuoteSnapshotValidator {
    pub rules: Vec<Rc<dyn QuoteSnapshotRule>>,
}

impl QuoteSnapshotValidator {
    pub fn validate(&self, item: &QuoteSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_snapshot_validator() -> QuoteSnapshotValidator {
    QuoteSnapshotValidator {
        rules: vec![
            Rc::new(QuoteSnapshotScopeLengthRule { max_length: 64 }) as Rc<dyn QuoteSnapshotRule>,
            Rc::new(QuoteSnapshotPinnedGuardRule { limit: 4.8 }),
            Rc::new(QuoteSnapshotKeyIdLengthRule { max_length: 80 }),
            Rc::new(QuoteSnapshotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(QuoteSnapshotCodeLengthRule { max_length: 255 }),
            Rc::new(QuoteSnapshotRequiredCodeRule {
                message: "quote snapshot needs a value".to_string(),
            }),
        ],
    }
}
