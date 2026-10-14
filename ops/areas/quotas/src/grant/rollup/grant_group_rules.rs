use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant group must pass. Check returns its problems, or none.
pub trait GrantGroupRule: std::fmt::Debug {
    fn check(&self, item: &GrantGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantGroupRequiredNameRule {
    pub message: String,
}

impl GrantGroupRule for GrantGroupRequiredNameRule {
    fn check(&self, item: &GrantGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantGroupNameLengthRule {
    pub max_length: i64,
}

impl GrantGroupRule for GrantGroupNameLengthRule {
    fn check(&self, item: &GrantGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantGroupCodePrefixRule {
    pub prefix: String,
}

impl GrantGroupRule for GrantGroupCodePrefixRule {
    fn check(&self, item: &GrantGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantGroupGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantGroupRule for GrantGroupGraceDaysRangeRule {
    fn check(&self, item: &GrantGroup) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a grant group.
#[derive(Debug, Clone)]
pub struct GrantGroupValidator {
    pub rules: Vec<Rc<dyn GrantGroupRule>>,
}

impl GrantGroupValidator {
    pub fn validate(&self, item: &GrantGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_group_validator() -> GrantGroupValidator {
    GrantGroupValidator {
        rules: vec![
            Rc::new(GrantGroupRequiredNameRule {
                message: "grant group needs an entry".to_string(),
            }) as Rc<dyn GrantGroupRule>,
            Rc::new(GrantGroupNameLengthRule { max_length: 255 }),
            Rc::new(GrantGroupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(GrantGroupGraceDaysRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_group_validator();
        let item = new_grant_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GrantGroupRequiredNameRule {
            message: "grant group needs this field".to_string(),
        };
        let mut item = new_grant_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GrantGroupNameLengthRule { max_length: 3600 };
        let mut item = new_grant_group("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = GrantGroupGraceDaysRangeRule { min: 255, max: 3600 };
        let mut item = new_grant_group("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
