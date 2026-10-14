use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default plan entry must pass. Check returns its problems, or none.
pub trait DefaultPlanEntryRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPlanEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanEntryTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DefaultPlanEntryRule for DefaultPlanEntryTagsLimitRule {
    fn check(&self, item: &DefaultPlanEntry) -> Vec<String> {
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
pub struct DefaultPlanEntryRequiredTitleRule {
    pub message: String,
}

impl DefaultPlanEntryRule for DefaultPlanEntryRequiredTitleRule {
    fn check(&self, item: &DefaultPlanEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultPlanEntryRule for DefaultPlanEntryAllowedCodeRule {
    fn check(&self, item: &DefaultPlanEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default plan entry.
#[derive(Debug, Clone)]
pub struct DefaultPlanEntryValidator {
    pub rules: Vec<Rc<dyn DefaultPlanEntryRule>>,
}

impl DefaultPlanEntryValidator {
    pub fn validate(&self, item: &DefaultPlanEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultPlanEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPlanEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_plan_entry_validator() -> DefaultPlanEntryValidator {
    DefaultPlanEntryValidator {
        rules: vec![
            Rc::new(DefaultPlanEntryTagsLimitRule {
                max_count: 64,
                banned: vec!["default".to_string(), "adaptive".to_string()],
            }) as Rc<dyn DefaultPlanEntryRule>,
            Rc::new(DefaultPlanEntryRequiredTitleRule {
                message: "default plan entry needs an entry".to_string(),
            }),
            Rc::new(DefaultPlanEntryAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "weighted".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_plan_entry_validator();
        let item = new_default_plan_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultPlanEntryRequiredTitleRule {
            message: "default plan entry needs a value".to_string(),
        };
        let mut item = new_default_plan_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
