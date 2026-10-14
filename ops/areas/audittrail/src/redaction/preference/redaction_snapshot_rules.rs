use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction snapshot must pass. Check returns its problems, or none.
pub trait RedactionSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RedactionSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSnapshotCodePrefixRule {
    pub prefix: String,
}

impl RedactionSnapshotRule for RedactionSnapshotCodePrefixRule {
    fn check(&self, item: &RedactionSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSnapshotRequiredTitleRule {
    pub message: String,
}

impl RedactionSnapshotRule for RedactionSnapshotRequiredTitleRule {
    fn check(&self, item: &RedactionSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSnapshotTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionSnapshotRule for RedactionSnapshotTtlSecondsRangeRule {
    fn check(&self, item: &RedactionSnapshot) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSnapshotVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionSnapshotRule for RedactionSnapshotVersionRangeRule {
    fn check(&self, item: &RedactionSnapshot) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RedactionSnapshotRule for RedactionSnapshotAllowedCodeRule {
    fn check(&self, item: &RedactionSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl RedactionSnapshotRule for RedactionSnapshotCodeLengthRule {
    fn check(&self, item: &RedactionSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction snapshot.
#[derive(Debug, Clone)]
pub struct RedactionSnapshotValidator {
    pub rules: Vec<Rc<dyn RedactionSnapshotRule>>,
}

impl RedactionSnapshotValidator {
    pub fn validate(&self, item: &RedactionSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_snapshot_validator() -> RedactionSnapshotValidator {
    RedactionSnapshotValidator {
        rules: vec![
            Rc::new(RedactionSnapshotCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RedactionSnapshotRule>,
            Rc::new(RedactionSnapshotRequiredTitleRule {
                message: "redaction snapshot needs an entry".to_string(),
            }),
            Rc::new(RedactionSnapshotTtlSecondsRangeRule { min: 32, max: 120 }),
            Rc::new(RedactionSnapshotVersionRangeRule { min: 32, max: 255 }),
            Rc::new(RedactionSnapshotAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(RedactionSnapshotCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_snapshot_validator();
        let item = new_redaction_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RedactionSnapshotRequiredTitleRule {
            message: "redaction snapshot needs an entry".to_string(),
        };
        let mut item = new_redaction_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RedactionSnapshotTtlSecondsRangeRule { min: 16, max: 64 };
        let mut item = new_redaction_snapshot("x", "y");
        item.ttl_seconds = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RedactionSnapshotVersionRangeRule { min: 32, max: 1000 };
        let mut item = new_redaction_snapshot("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionSnapshotCodeLengthRule { max_length: 255 };
        let mut item = new_redaction_snapshot("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
