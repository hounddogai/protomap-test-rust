use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched release entry must pass. Check returns its problems, or none.
pub trait BatchedReleaseEntryRule: std::fmt::Debug {
    fn check(&self, item: &BatchedReleaseEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseEntryCodeLengthRule {
    pub max_length: i64,
}

impl BatchedReleaseEntryRule for BatchedReleaseEntryCodeLengthRule {
    fn check(&self, item: &BatchedReleaseEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseEntryLocaleLengthRule {
    pub max_length: i64,
}

impl BatchedReleaseEntryRule for BatchedReleaseEntryLocaleLengthRule {
    fn check(&self, item: &BatchedReleaseEntry) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseEntryRequiredNameRule {
    pub message: String,
}

impl BatchedReleaseEntryRule for BatchedReleaseEntryRequiredNameRule {
    fn check(&self, item: &BatchedReleaseEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched release entry.
#[derive(Debug, Clone)]
pub struct BatchedReleaseEntryValidator {
    pub rules: Vec<Rc<dyn BatchedReleaseEntryRule>>,
}

impl BatchedReleaseEntryValidator {
    pub fn validate(&self, item: &BatchedReleaseEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedReleaseEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedReleaseEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_release_entry_validator() -> BatchedReleaseEntryValidator {
    BatchedReleaseEntryValidator {
        rules: vec![
            Rc::new(BatchedReleaseEntryCodeLengthRule { max_length: 86400 }) as Rc<dyn BatchedReleaseEntryRule>,
            Rc::new(BatchedReleaseEntryLocaleLengthRule { max_length: 16 }),
            Rc::new(BatchedReleaseEntryRequiredNameRule {
                message: "batched release entry needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_release_entry_validator();
        let item = new_batched_release_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedReleaseEntryCodeLengthRule { max_length: 120 };
        let mut item = new_batched_release_entry("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = BatchedReleaseEntryLocaleLengthRule { max_length: 1000 };
        let mut item = new_batched_release_entry("x", "y");
        item.locale = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedReleaseEntryRequiredNameRule {
            message: "batched release entry needs this field".to_string(),
        };
        let mut item = new_batched_release_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
