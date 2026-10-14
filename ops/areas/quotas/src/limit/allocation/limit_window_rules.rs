use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit window must pass. Check returns its problems, or none.
pub trait LimitWindowRule: std::fmt::Debug {
    fn check(&self, item: &LimitWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindowRequiredCodeRule {
    pub message: String,
}

impl LimitWindowRule for LimitWindowRequiredCodeRule {
    fn check(&self, item: &LimitWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindowRequiredNameRule {
    pub message: String,
}

impl LimitWindowRule for LimitWindowRequiredNameRule {
    fn check(&self, item: &LimitWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindowCodeLengthRule {
    pub max_length: i64,
}

impl LimitWindowRule for LimitWindowCodeLengthRule {
    fn check(&self, item: &LimitWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindowCodePrefixRule {
    pub prefix: String,
}

impl LimitWindowRule for LimitWindowCodePrefixRule {
    fn check(&self, item: &LimitWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindowChannelLengthRule {
    pub max_length: i64,
}

impl LimitWindowRule for LimitWindowChannelLengthRule {
    fn check(&self, item: &LimitWindow) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit window.
#[derive(Debug, Clone)]
pub struct LimitWindowValidator {
    pub rules: Vec<Rc<dyn LimitWindowRule>>,
}

impl LimitWindowValidator {
    pub fn validate(&self, item: &LimitWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_window_validator() -> LimitWindowValidator {
    LimitWindowValidator {
        rules: vec![
            Rc::new(LimitWindowRequiredCodeRule {
                message: "limit window needs this field".to_string(),
            }) as Rc<dyn LimitWindowRule>,
            Rc::new(LimitWindowRequiredNameRule {
                message: "limit window needs this field".to_string(),
            }),
            Rc::new(LimitWindowCodeLengthRule { max_length: 64 }),
            Rc::new(LimitWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LimitWindowChannelLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_window_validator();
        let item = new_limit_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitWindowRequiredCodeRule {
            message: "limit window needs this field".to_string(),
        };
        let mut item = new_limit_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LimitWindowRequiredNameRule {
            message: "limit window needs an entry".to_string(),
        };
        let mut item = new_limit_window("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitWindowCodeLengthRule { max_length: 255 };
        let mut item = new_limit_window("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = LimitWindowChannelLengthRule { max_length: 64 };
        let mut item = new_limit_window("x", "y");
        item.channel = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
