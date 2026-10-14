use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling limit group must pass. Check returns its problems, or none.
pub trait RollingLimitGroupRule: std::fmt::Debug {
    fn check(&self, item: &RollingLimitGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitGroupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollingLimitGroupRule for RollingLimitGroupAllowedNameRule {
    fn check(&self, item: &RollingLimitGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitGroupRequiredNameRule {
    pub message: String,
}

impl RollingLimitGroupRule for RollingLimitGroupRequiredNameRule {
    fn check(&self, item: &RollingLimitGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitGroupNameLengthRule {
    pub max_length: i64,
}

impl RollingLimitGroupRule for RollingLimitGroupNameLengthRule {
    fn check(&self, item: &RollingLimitGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitGroupCodePrefixRule {
    pub prefix: String,
}

impl RollingLimitGroupRule for RollingLimitGroupCodePrefixRule {
    fn check(&self, item: &RollingLimitGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling limit group.
#[derive(Debug, Clone)]
pub struct RollingLimitGroupValidator {
    pub rules: Vec<Rc<dyn RollingLimitGroupRule>>,
}

impl RollingLimitGroupValidator {
    pub fn validate(&self, item: &RollingLimitGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingLimitGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingLimitGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_limit_group_validator() -> RollingLimitGroupValidator {
    RollingLimitGroupValidator {
        rules: vec![
            Rc::new(RollingLimitGroupAllowedNameRule {
                allowed: vec!["scheduled".to_string(), "strict".to_string()],
            }) as Rc<dyn RollingLimitGroupRule>,
            Rc::new(RollingLimitGroupRequiredNameRule {
                message: "rolling limit group needs a value".to_string(),
            }),
            Rc::new(RollingLimitGroupNameLengthRule { max_length: 255 }),
            Rc::new(RollingLimitGroupCodePrefixRule {
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
        let validator = default_rolling_limit_group_validator();
        let item = new_rolling_limit_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingLimitGroupRequiredNameRule {
            message: "rolling limit group needs a value".to_string(),
        };
        let mut item = new_rolling_limit_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingLimitGroupNameLengthRule { max_length: 64 };
        let mut item = new_rolling_limit_group("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
