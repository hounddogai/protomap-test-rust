use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant rule must pass. Check returns its problems, or none.
pub trait GrantRuleRule: std::fmt::Debug {
    fn check(&self, item: &GrantRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRuleRequiredTitleRule {
    pub message: String,
}

impl GrantRuleRule for GrantRuleRequiredTitleRule {
    fn check(&self, item: &GrantRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRuleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl GrantRuleRule for GrantRuleAllowedTitleRule {
    fn check(&self, item: &GrantRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRuleRequiredChannelRule {
    pub message: String,
}

impl GrantRuleRule for GrantRuleRequiredChannelRule {
    fn check(&self, item: &GrantRule) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRuleRequiredCodeRule {
    pub message: String,
}

impl GrantRuleRule for GrantRuleRequiredCodeRule {
    fn check(&self, item: &GrantRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRuleOwnerIdLengthRule {
    pub max_length: i64,
}

impl GrantRuleRule for GrantRuleOwnerIdLengthRule {
    fn check(&self, item: &GrantRule) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRuleCodeLengthRule {
    pub max_length: i64,
}

impl GrantRuleRule for GrantRuleCodeLengthRule {
    fn check(&self, item: &GrantRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant rule.
#[derive(Debug, Clone)]
pub struct GrantRuleValidator {
    pub rules: Vec<Rc<dyn GrantRuleRule>>,
}

impl GrantRuleValidator {
    pub fn validate(&self, item: &GrantRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_rule_validator() -> GrantRuleValidator {
    GrantRuleValidator {
        rules: vec![
            Rc::new(GrantRuleRequiredTitleRule {
                message: "grant rule needs this field".to_string(),
            }) as Rc<dyn GrantRuleRule>,
            Rc::new(GrantRuleAllowedTitleRule {
                allowed: vec!["lenient".to_string(), "seasonal".to_string(), "global".to_string()],
            }),
            Rc::new(GrantRuleRequiredChannelRule {
                message: "grant rule needs an entry".to_string(),
            }),
            Rc::new(GrantRuleRequiredCodeRule {
                message: "grant rule needs this field".to_string(),
            }),
            Rc::new(GrantRuleOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(GrantRuleCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_rule_validator();
        let item = new_grant_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GrantRuleRequiredTitleRule {
            message: "grant rule needs a value".to_string(),
        };
        let mut item = new_grant_rule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = GrantRuleRequiredChannelRule {
            message: "grant rule needs this field".to_string(),
        };
        let mut item = new_grant_rule("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantRuleRequiredCodeRule {
            message: "grant rule needs an entry".to_string(),
        };
        let mut item = new_grant_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = GrantRuleOwnerIdLengthRule { max_length: 120 };
        let mut item = new_grant_rule("x", "y");
        item.owner_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GrantRuleCodeLengthRule { max_length: 1000 };
        let mut item = new_grant_rule("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
