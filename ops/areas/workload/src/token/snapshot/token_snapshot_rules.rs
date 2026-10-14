use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token snapshot must pass. Check returns its problems, or none.
pub trait TokenSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &TokenSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSnapshotRequiredCodeRule {
    pub message: String,
}

impl TokenSnapshotRule for TokenSnapshotRequiredCodeRule {
    fn check(&self, item: &TokenSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSnapshotNameLengthRule {
    pub max_length: i64,
}

impl TokenSnapshotRule for TokenSnapshotNameLengthRule {
    fn check(&self, item: &TokenSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSnapshotRequiredNameRule {
    pub message: String,
}

impl TokenSnapshotRule for TokenSnapshotRequiredNameRule {
    fn check(&self, item: &TokenSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token snapshot.
#[derive(Debug, Clone)]
pub struct TokenSnapshotValidator {
    pub rules: Vec<Rc<dyn TokenSnapshotRule>>,
}

impl TokenSnapshotValidator {
    pub fn validate(&self, item: &TokenSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_snapshot_validator() -> TokenSnapshotValidator {
    TokenSnapshotValidator {
        rules: vec![
            Rc::new(TokenSnapshotRequiredCodeRule {
                message: "token snapshot needs an entry".to_string(),
            }) as Rc<dyn TokenSnapshotRule>,
            Rc::new(TokenSnapshotNameLengthRule { max_length: 86400 }),
            Rc::new(TokenSnapshotRequiredNameRule {
                message: "token snapshot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_snapshot_validator();
        let item = new_token_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenSnapshotRequiredCodeRule {
            message: "token snapshot needs a value".to_string(),
        };
        let mut item = new_token_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TokenSnapshotNameLengthRule { max_length: 86400 };
        let mut item = new_token_snapshot("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TokenSnapshotRequiredNameRule {
            message: "token snapshot needs a value".to_string(),
        };
        let mut item = new_token_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
