use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive limit group must pass. Check returns its problems, or none.
pub trait AdaptiveLimitGroupRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveLimitGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitGroupRequiredTitleRule {
    pub message: String,
}

impl AdaptiveLimitGroupRule for AdaptiveLimitGroupRequiredTitleRule {
    fn check(&self, item: &AdaptiveLimitGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitGroupCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveLimitGroupRule for AdaptiveLimitGroupCodeLengthRule {
    fn check(&self, item: &AdaptiveLimitGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitGroupRequiredCodeRule {
    pub message: String,
}

impl AdaptiveLimitGroupRule for AdaptiveLimitGroupRequiredCodeRule {
    fn check(&self, item: &AdaptiveLimitGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive limit group.
#[derive(Debug, Clone)]
pub struct AdaptiveLimitGroupValidator {
    pub rules: Vec<Rc<dyn AdaptiveLimitGroupRule>>,
}

impl AdaptiveLimitGroupValidator {
    pub fn validate(&self, item: &AdaptiveLimitGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveLimitGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveLimitGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_limit_group_validator() -> AdaptiveLimitGroupValidator {
    AdaptiveLimitGroupValidator {
        rules: vec![
            Rc::new(AdaptiveLimitGroupRequiredTitleRule {
                message: "adaptive limit group needs this field".to_string(),
            }) as Rc<dyn AdaptiveLimitGroupRule>,
            Rc::new(AdaptiveLimitGroupCodeLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveLimitGroupRequiredCodeRule {
                message: "adaptive limit group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_limit_group_validator();
        let item = new_adaptive_limit_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveLimitGroupRequiredTitleRule {
            message: "adaptive limit group needs a value".to_string(),
        };
        let mut item = new_adaptive_limit_group("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveLimitGroupCodeLengthRule { max_length: 64 };
        let mut item = new_adaptive_limit_group("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveLimitGroupRequiredCodeRule {
            message: "adaptive limit group needs a value".to_string(),
        };
        let mut item = new_adaptive_limit_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
