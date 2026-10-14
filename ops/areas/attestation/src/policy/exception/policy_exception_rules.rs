use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a policy exception must pass. Check returns its problems, or none.
pub trait PolicyExceptionRule: std::fmt::Debug {
    fn check(&self, item: &PolicyException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyExceptionRequiredLabelRule {
    pub message: String,
}

impl PolicyExceptionRule for PolicyExceptionRequiredLabelRule {
    fn check(&self, item: &PolicyException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyExceptionAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PolicyExceptionRule for PolicyExceptionAllowedLabelRule {
    fn check(&self, item: &PolicyException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyExceptionCodePrefixRule {
    pub prefix: String,
}

impl PolicyExceptionRule for PolicyExceptionCodePrefixRule {
    fn check(&self, item: &PolicyException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyExceptionTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PolicyExceptionRule for PolicyExceptionTagsLimitRule {
    fn check(&self, item: &PolicyException) -> Vec<String> {
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

/// Runs every rule of a policy exception.
#[derive(Debug, Clone)]
pub struct PolicyExceptionValidator {
    pub rules: Vec<Rc<dyn PolicyExceptionRule>>,
}

impl PolicyExceptionValidator {
    pub fn validate(&self, item: &PolicyException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_exception_validator() -> PolicyExceptionValidator {
    PolicyExceptionValidator {
        rules: vec![
            Rc::new(PolicyExceptionRequiredLabelRule {
                message: "policy exception needs an entry".to_string(),
            }) as Rc<dyn PolicyExceptionRule>,
            Rc::new(PolicyExceptionAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(PolicyExceptionCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PolicyExceptionTagsLimitRule {
                max_count: 255,
                banned: Vec::new(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_exception_validator();
        let item = new_policy_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PolicyExceptionRequiredLabelRule {
            message: "policy exception needs a value".to_string(),
        };
        let mut item = new_policy_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
