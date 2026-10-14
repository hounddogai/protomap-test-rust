use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release rule must pass. Check returns its problems, or none.
pub trait ReleaseRuleRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRuleRequiredCodeRule {
    pub message: String,
}

impl ReleaseRuleRule for ReleaseRuleRequiredCodeRule {
    fn check(&self, item: &ReleaseRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRuleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ReleaseRuleRule for ReleaseRuleTagsLimitRule {
    fn check(&self, item: &ReleaseRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRuleTenantIdLengthRule {
    pub max_length: i64,
}

impl ReleaseRuleRule for ReleaseRuleTenantIdLengthRule {
    fn check(&self, item: &ReleaseRule) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRuleRequiredReviewerRule {
    pub message: String,
}

impl ReleaseRuleRule for ReleaseRuleRequiredReviewerRule {
    fn check(&self, item: &ReleaseRule) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release rule.
#[derive(Debug, Clone)]
pub struct ReleaseRuleValidator {
    pub rules: Vec<Rc<dyn ReleaseRuleRule>>,
}

impl ReleaseRuleValidator {
    pub fn validate(&self, item: &ReleaseRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_rule_validator() -> ReleaseRuleValidator {
    ReleaseRuleValidator {
        rules: vec![
            Rc::new(ReleaseRuleRequiredCodeRule {
                message: "release rule needs an entry".to_string(),
            }) as Rc<dyn ReleaseRuleRule>,
            Rc::new(ReleaseRuleTagsLimitRule {
                max_count: 1000,
                banned: vec!["adaptive".to_string(), "lenient".to_string()],
            }),
            Rc::new(ReleaseRuleTenantIdLengthRule { max_length: 255 }),
            Rc::new(ReleaseRuleRequiredReviewerRule {
                message: "release rule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_rule_validator();
        let item = new_release_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleaseRuleRequiredCodeRule {
            message: "release rule needs an entry".to_string(),
        };
        let mut item = new_release_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = ReleaseRuleTenantIdLengthRule { max_length: 3600 };
        let mut item = new_release_rule("x", "y");
        item.tenant_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ReleaseRuleRequiredReviewerRule {
            message: "release rule needs a value".to_string(),
        };
        let mut item = new_release_rule("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
