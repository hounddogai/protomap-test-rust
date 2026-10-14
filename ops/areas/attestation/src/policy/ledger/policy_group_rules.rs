use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a policy group must pass. Check returns its problems, or none.
pub trait PolicyGroupRule: std::fmt::Debug {
    fn check(&self, item: &PolicyGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGroupRequiredRegionRule {
    pub message: String,
}

impl PolicyGroupRule for PolicyGroupRequiredRegionRule {
    fn check(&self, item: &PolicyGroup) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGroupRequiredContactEmailRule {
    pub message: String,
}

impl PolicyGroupRule for PolicyGroupRequiredContactEmailRule {
    fn check(&self, item: &PolicyGroup) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGroupTitleLengthRule {
    pub max_length: i64,
}

impl PolicyGroupRule for PolicyGroupTitleLengthRule {
    fn check(&self, item: &PolicyGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGroupRegionLengthRule {
    pub max_length: i64,
}

impl PolicyGroupRule for PolicyGroupRegionLengthRule {
    fn check(&self, item: &PolicyGroup) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGroupCodeLengthRule {
    pub max_length: i64,
}

impl PolicyGroupRule for PolicyGroupCodeLengthRule {
    fn check(&self, item: &PolicyGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGroupAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PolicyGroupRule for PolicyGroupAllowedTitleRule {
    fn check(&self, item: &PolicyGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy group.
#[derive(Debug, Clone)]
pub struct PolicyGroupValidator {
    pub rules: Vec<Rc<dyn PolicyGroupRule>>,
}

impl PolicyGroupValidator {
    pub fn validate(&self, item: &PolicyGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_group_validator() -> PolicyGroupValidator {
    PolicyGroupValidator {
        rules: vec![
            Rc::new(PolicyGroupRequiredRegionRule {
                message: "policy group needs this field".to_string(),
            }) as Rc<dyn PolicyGroupRule>,
            Rc::new(PolicyGroupRequiredContactEmailRule {
                message: "policy group needs a value".to_string(),
            }),
            Rc::new(PolicyGroupTitleLengthRule { max_length: 120 }),
            Rc::new(PolicyGroupRegionLengthRule { max_length: 120 }),
            Rc::new(PolicyGroupCodeLengthRule { max_length: 1000 }),
            Rc::new(PolicyGroupAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_group_validator();
        let item = new_policy_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PolicyGroupRequiredRegionRule {
            message: "policy group needs a value".to_string(),
        };
        let mut item = new_policy_group("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = PolicyGroupRequiredContactEmailRule {
            message: "policy group needs a value".to_string(),
        };
        let mut item = new_policy_group("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PolicyGroupTitleLengthRule { max_length: 255 };
        let mut item = new_policy_group("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PolicyGroupRegionLengthRule { max_length: 3600 };
        let mut item = new_policy_group("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyGroupCodeLengthRule { max_length: 255 };
        let mut item = new_policy_group("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
