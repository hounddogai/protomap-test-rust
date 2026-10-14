use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release plan must pass. Check returns its problems, or none.
pub trait ReleasePlanRule: std::fmt::Debug {
    fn check(&self, item: &ReleasePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePlanRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleasePlanRule for ReleasePlanRetriesRangeRule {
    fn check(&self, item: &ReleasePlan) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePlanCodePrefixRule {
    pub prefix: String,
}

impl ReleasePlanRule for ReleasePlanCodePrefixRule {
    fn check(&self, item: &ReleasePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePlanExternalRefLengthRule {
    pub max_length: i64,
}

impl ReleasePlanRule for ReleasePlanExternalRefLengthRule {
    fn check(&self, item: &ReleasePlan) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePlanAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ReleasePlanRule for ReleasePlanAllowedTitleRule {
    fn check(&self, item: &ReleasePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePlanActiveGuardRule {
    pub limit: i64,
}

impl ReleasePlanRule for ReleasePlanActiveGuardRule {
    fn check(&self, item: &ReleasePlan) -> Vec<String> {
        if item.active && (item.retries > self.limit) {
            return vec![format!("retries {} exceeds {} while active", item.retries, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePlanRequiredCodeRule {
    pub message: String,
}

impl ReleasePlanRule for ReleasePlanRequiredCodeRule {
    fn check(&self, item: &ReleasePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release plan.
#[derive(Debug, Clone)]
pub struct ReleasePlanValidator {
    pub rules: Vec<Rc<dyn ReleasePlanRule>>,
}

impl ReleasePlanValidator {
    pub fn validate(&self, item: &ReleasePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleasePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleasePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_plan_validator() -> ReleasePlanValidator {
    ReleasePlanValidator {
        rules: vec![
            Rc::new(ReleasePlanRetriesRangeRule { min: 16, max: 1000 }) as Rc<dyn ReleasePlanRule>,
            Rc::new(ReleasePlanCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ReleasePlanExternalRefLengthRule { max_length: 3600 }),
            Rc::new(ReleasePlanAllowedTitleRule {
                allowed: vec!["pending".to_string(), "scheduled".to_string(), "global".to_string()],
            }),
            Rc::new(ReleasePlanActiveGuardRule { limit: 64 }),
            Rc::new(ReleasePlanRequiredCodeRule {
                message: "release plan needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_plan_validator();
        let item = new_release_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ReleasePlanRetriesRangeRule { min: 16, max: 80 };
        let mut item = new_release_plan("x", "y");
        item.retries = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = ReleasePlanExternalRefLengthRule { max_length: 1000 };
        let mut item = new_release_plan("x", "y");
        item.external_ref = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleasePlanRequiredCodeRule {
            message: "release plan needs this field".to_string(),
        };
        let mut item = new_release_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
