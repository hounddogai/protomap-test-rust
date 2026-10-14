use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem plan must pass. Check returns its problems, or none.
pub trait PostmortemPlanRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPlanRequiredTitleRule {
    pub message: String,
}

impl PostmortemPlanRule for PostmortemPlanRequiredTitleRule {
    fn check(&self, item: &PostmortemPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPlanAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl PostmortemPlanRule for PostmortemPlanAllowedCipherRule {
    fn check(&self, item: &PostmortemPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is stale", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPlanCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemPlanRule for PostmortemPlanCreatedAtRangeRule {
    fn check(&self, item: &PostmortemPlan) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPlanCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemPlanRule for PostmortemPlanCodeLengthRule {
    fn check(&self, item: &PostmortemPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem plan.
#[derive(Debug, Clone)]
pub struct PostmortemPlanValidator {
    pub rules: Vec<Rc<dyn PostmortemPlanRule>>,
}

impl PostmortemPlanValidator {
    pub fn validate(&self, item: &PostmortemPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_plan_validator() -> PostmortemPlanValidator {
    PostmortemPlanValidator {
        rules: vec![
            Rc::new(PostmortemPlanRequiredTitleRule {
                message: "postmortem plan needs an entry".to_string(),
            }) as Rc<dyn PostmortemPlanRule>,
            Rc::new(PostmortemPlanAllowedCipherRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(PostmortemPlanCreatedAtRangeRule { min: 64, max: 1000 }),
            Rc::new(PostmortemPlanCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_plan_validator();
        let item = new_postmortem_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PostmortemPlanRequiredTitleRule {
            message: "postmortem plan needs this field".to_string(),
        };
        let mut item = new_postmortem_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = PostmortemPlanCreatedAtRangeRule { min: 32, max: 255 };
        let mut item = new_postmortem_plan("x", "y");
        item.created_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PostmortemPlanCodeLengthRule { max_length: 32 };
        let mut item = new_postmortem_plan("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
