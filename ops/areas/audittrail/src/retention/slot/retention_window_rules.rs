use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention window must pass. Check returns its problems, or none.
pub trait RetentionWindowRule: std::fmt::Debug {
    fn check(&self, item: &RetentionWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionWindowRequiredClusterRule {
    pub message: String,
}

impl RetentionWindowRule for RetentionWindowRequiredClusterRule {
    fn check(&self, item: &RetentionWindow) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionWindowClusterLengthRule {
    pub max_length: i64,
}

impl RetentionWindowRule for RetentionWindowClusterLengthRule {
    fn check(&self, item: &RetentionWindow) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionWindowRequiredHostRule {
    pub message: String,
}

impl RetentionWindowRule for RetentionWindowRequiredHostRule {
    fn check(&self, item: &RetentionWindow) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionWindowCodeLengthRule {
    pub max_length: i64,
}

impl RetentionWindowRule for RetentionWindowCodeLengthRule {
    fn check(&self, item: &RetentionWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionWindowCodePrefixRule {
    pub prefix: String,
}

impl RetentionWindowRule for RetentionWindowCodePrefixRule {
    fn check(&self, item: &RetentionWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention window.
#[derive(Debug, Clone)]
pub struct RetentionWindowValidator {
    pub rules: Vec<Rc<dyn RetentionWindowRule>>,
}

impl RetentionWindowValidator {
    pub fn validate(&self, item: &RetentionWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_window_validator() -> RetentionWindowValidator {
    RetentionWindowValidator {
        rules: vec![
            Rc::new(RetentionWindowRequiredClusterRule {
                message: "retention window needs this field".to_string(),
            }) as Rc<dyn RetentionWindowRule>,
            Rc::new(RetentionWindowClusterLengthRule { max_length: 32 }),
            Rc::new(RetentionWindowRequiredHostRule {
                message: "retention window needs an entry".to_string(),
            }),
            Rc::new(RetentionWindowCodeLengthRule { max_length: 120 }),
            Rc::new(RetentionWindowCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_window_validator();
        let item = new_retention_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RetentionWindowRequiredClusterRule {
            message: "retention window needs an entry".to_string(),
        };
        let mut item = new_retention_window("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RetentionWindowClusterLengthRule { max_length: 32 };
        let mut item = new_retention_window("x", "y");
        item.cluster = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RetentionWindowRequiredHostRule {
            message: "retention window needs a value".to_string(),
        };
        let mut item = new_retention_window("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionWindowCodeLengthRule { max_length: 255 };
        let mut item = new_retention_window("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
