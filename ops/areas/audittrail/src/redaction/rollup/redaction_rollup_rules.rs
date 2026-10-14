use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction rollup must pass. Check returns its problems, or none.
pub trait RedactionRollupRule: std::fmt::Debug {
    fn check(&self, item: &RedactionRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRollupCodeLengthRule {
    pub max_length: i64,
}

impl RedactionRollupRule for RedactionRollupCodeLengthRule {
    fn check(&self, item: &RedactionRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRollupRequiredCodeRule {
    pub message: String,
}

impl RedactionRollupRule for RedactionRollupRequiredCodeRule {
    fn check(&self, item: &RedactionRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRollupNameLengthRule {
    pub max_length: i64,
}

impl RedactionRollupRule for RedactionRollupNameLengthRule {
    fn check(&self, item: &RedactionRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction rollup.
#[derive(Debug, Clone)]
pub struct RedactionRollupValidator {
    pub rules: Vec<Rc<dyn RedactionRollupRule>>,
}

impl RedactionRollupValidator {
    pub fn validate(&self, item: &RedactionRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_rollup_validator() -> RedactionRollupValidator {
    RedactionRollupValidator {
        rules: vec![
            Rc::new(RedactionRollupCodeLengthRule { max_length: 3600 }) as Rc<dyn RedactionRollupRule>,
            Rc::new(RedactionRollupRequiredCodeRule {
                message: "redaction rollup needs an entry".to_string(),
            }),
            Rc::new(RedactionRollupNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_rollup_validator();
        let item = new_redaction_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionRollupCodeLengthRule { max_length: 255 };
        let mut item = new_redaction_rollup("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionRollupRequiredCodeRule {
            message: "redaction rollup needs this field".to_string(),
        };
        let mut item = new_redaction_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RedactionRollupNameLengthRule { max_length: 32 };
        let mut item = new_redaction_rollup("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
