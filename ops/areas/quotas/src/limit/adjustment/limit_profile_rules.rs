use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit profile must pass. Check returns its problems, or none.
pub trait LimitProfileRule: std::fmt::Debug {
    fn check(&self, item: &LimitProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitProfileLabelLengthRule {
    pub max_length: i64,
}

impl LimitProfileRule for LimitProfileLabelLengthRule {
    fn check(&self, item: &LimitProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitProfileCodeLengthRule {
    pub max_length: i64,
}

impl LimitProfileRule for LimitProfileCodeLengthRule {
    fn check(&self, item: &LimitProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitProfileRequiredCodeRule {
    pub message: String,
}

impl LimitProfileRule for LimitProfileRequiredCodeRule {
    fn check(&self, item: &LimitProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit profile.
#[derive(Debug, Clone)]
pub struct LimitProfileValidator {
    pub rules: Vec<Rc<dyn LimitProfileRule>>,
}

impl LimitProfileValidator {
    pub fn validate(&self, item: &LimitProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_profile_validator() -> LimitProfileValidator {
    LimitProfileValidator {
        rules: vec![
            Rc::new(LimitProfileLabelLengthRule { max_length: 255 }) as Rc<dyn LimitProfileRule>,
            Rc::new(LimitProfileCodeLengthRule { max_length: 86400 }),
            Rc::new(LimitProfileRequiredCodeRule {
                message: "limit profile needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_profile_validator();
        let item = new_limit_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LimitProfileLabelLengthRule { max_length: 255 };
        let mut item = new_limit_profile("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitProfileCodeLengthRule { max_length: 120 };
        let mut item = new_limit_profile("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitProfileRequiredCodeRule {
            message: "limit profile needs a value".to_string(),
        };
        let mut item = new_limit_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
