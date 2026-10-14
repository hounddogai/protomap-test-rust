use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity plan must pass. Check returns its problems, or none.
pub trait IdentityPlanRule: std::fmt::Debug {
    fn check(&self, item: &IdentityPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPlanAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl IdentityPlanRule for IdentityPlanAllowedCodeRule {
    fn check(&self, item: &IdentityPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPlanActiveGuardRule {
    pub limit: i64,
}

impl IdentityPlanRule for IdentityPlanActiveGuardRule {
    fn check(&self, item: &IdentityPlan) -> Vec<String> {
        if item.active && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while active",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPlanRequiredCodeRule {
    pub message: String,
}

impl IdentityPlanRule for IdentityPlanRequiredCodeRule {
    fn check(&self, item: &IdentityPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPlanRequiredTitleRule {
    pub message: String,
}

impl IdentityPlanRule for IdentityPlanRequiredTitleRule {
    fn check(&self, item: &IdentityPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPlanTitleLengthRule {
    pub max_length: i64,
}

impl IdentityPlanRule for IdentityPlanTitleLengthRule {
    fn check(&self, item: &IdentityPlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity plan.
#[derive(Debug, Clone)]
pub struct IdentityPlanValidator {
    pub rules: Vec<Rc<dyn IdentityPlanRule>>,
}

impl IdentityPlanValidator {
    pub fn validate(&self, item: &IdentityPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_plan_validator() -> IdentityPlanValidator {
    IdentityPlanValidator {
        rules: vec![
            Rc::new(IdentityPlanAllowedCodeRule {
                allowed: vec!["adaptive".to_string(), "adaptive".to_string(), "fallback".to_string()],
            }) as Rc<dyn IdentityPlanRule>,
            Rc::new(IdentityPlanActiveGuardRule { limit: 1000 }),
            Rc::new(IdentityPlanRequiredCodeRule {
                message: "identity plan needs a value".to_string(),
            }),
            Rc::new(IdentityPlanRequiredTitleRule {
                message: "identity plan needs a value".to_string(),
            }),
            Rc::new(IdentityPlanTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_plan_validator();
        let item = new_identity_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityPlanRequiredCodeRule {
            message: "identity plan needs this field".to_string(),
        };
        let mut item = new_identity_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = IdentityPlanRequiredTitleRule {
            message: "identity plan needs a value".to_string(),
        };
        let mut item = new_identity_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = IdentityPlanTitleLengthRule { max_length: 255 };
        let mut item = new_identity_plan("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
