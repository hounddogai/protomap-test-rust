use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled policy summary must pass. Check returns its problems, or none.
pub trait ScheduledPolicySummaryRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPolicySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicySummaryCategoryLengthRule {
    pub max_length: i64,
}

impl ScheduledPolicySummaryRule for ScheduledPolicySummaryCategoryLengthRule {
    fn check(&self, item: &ScheduledPolicySummary) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicySummaryRequiredCategoryRule {
    pub message: String,
}

impl ScheduledPolicySummaryRule for ScheduledPolicySummaryRequiredCategoryRule {
    fn check(&self, item: &ScheduledPolicySummary) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicySummaryLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledPolicySummaryRule for ScheduledPolicySummaryLabelLengthRule {
    fn check(&self, item: &ScheduledPolicySummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled policy summary.
#[derive(Debug, Clone)]
pub struct ScheduledPolicySummaryValidator {
    pub rules: Vec<Rc<dyn ScheduledPolicySummaryRule>>,
}

impl ScheduledPolicySummaryValidator {
    pub fn validate(&self, item: &ScheduledPolicySummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledPolicySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPolicySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_policy_summary_validator() -> ScheduledPolicySummaryValidator {
    ScheduledPolicySummaryValidator {
        rules: vec![
            Rc::new(ScheduledPolicySummaryCategoryLengthRule { max_length: 1000 })
                as Rc<dyn ScheduledPolicySummaryRule>,
            Rc::new(ScheduledPolicySummaryRequiredCategoryRule {
                message: "scheduled policy summary needs an entry".to_string(),
            }),
            Rc::new(ScheduledPolicySummaryLabelLengthRule { max_length: 3600 }),
        ],
    }
}
