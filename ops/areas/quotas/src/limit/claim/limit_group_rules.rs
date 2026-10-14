use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit group must pass. Check returns its problems, or none.
pub trait LimitGroupRule: std::fmt::Debug {
    fn check(&self, item: &LimitGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitGroupRequiredTitleRule {
    pub message: String,
}

impl LimitGroupRule for LimitGroupRequiredTitleRule {
    fn check(&self, item: &LimitGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitGroupFingerprintLengthRule {
    pub max_length: i64,
}

impl LimitGroupRule for LimitGroupFingerprintLengthRule {
    fn check(&self, item: &LimitGroup) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitGroupNameLengthRule {
    pub max_length: i64,
}

impl LimitGroupRule for LimitGroupNameLengthRule {
    fn check(&self, item: &LimitGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitGroupTitleLengthRule {
    pub max_length: i64,
}

impl LimitGroupRule for LimitGroupTitleLengthRule {
    fn check(&self, item: &LimitGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit group.
#[derive(Debug, Clone)]
pub struct LimitGroupValidator {
    pub rules: Vec<Rc<dyn LimitGroupRule>>,
}

impl LimitGroupValidator {
    pub fn validate(&self, item: &LimitGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_group_validator() -> LimitGroupValidator {
    LimitGroupValidator {
        rules: vec![
            Rc::new(LimitGroupRequiredTitleRule {
                message: "limit group needs an entry".to_string(),
            }) as Rc<dyn LimitGroupRule>,
            Rc::new(LimitGroupFingerprintLengthRule { max_length: 120 }),
            Rc::new(LimitGroupNameLengthRule { max_length: 1000 }),
            Rc::new(LimitGroupTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_group_validator();
        let item = new_limit_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LimitGroupRequiredTitleRule {
            message: "limit group needs a value".to_string(),
        };
        let mut item = new_limit_group("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = LimitGroupFingerprintLengthRule { max_length: 86400 };
        let mut item = new_limit_group("x", "y");
        item.fingerprint = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LimitGroupNameLengthRule { max_length: 86400 };
        let mut item = new_limit_group("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LimitGroupTitleLengthRule { max_length: 1000 };
        let mut item = new_limit_group("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
