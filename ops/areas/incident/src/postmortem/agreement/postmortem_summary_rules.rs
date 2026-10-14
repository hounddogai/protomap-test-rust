use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem summary must pass. Check returns its problems, or none.
pub trait PostmortemSummaryRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSummaryRequiredLabelRule {
    pub message: String,
}

impl PostmortemSummaryRule for PostmortemSummaryRequiredLabelRule {
    fn check(&self, item: &PostmortemSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSummaryCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemSummaryRule for PostmortemSummaryCodeLengthRule {
    fn check(&self, item: &PostmortemSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSummaryCountryLengthRule {
    pub max_length: i64,
}

impl PostmortemSummaryRule for PostmortemSummaryCountryLengthRule {
    fn check(&self, item: &PostmortemSummary) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSummaryRequiredCountryRule {
    pub message: String,
}

impl PostmortemSummaryRule for PostmortemSummaryRequiredCountryRule {
    fn check(&self, item: &PostmortemSummary) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem summary.
#[derive(Debug, Clone)]
pub struct PostmortemSummaryValidator {
    pub rules: Vec<Rc<dyn PostmortemSummaryRule>>,
}

impl PostmortemSummaryValidator {
    pub fn validate(&self, item: &PostmortemSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_summary_validator() -> PostmortemSummaryValidator {
    PostmortemSummaryValidator {
        rules: vec![
            Rc::new(PostmortemSummaryRequiredLabelRule {
                message: "postmortem summary needs a value".to_string(),
            }) as Rc<dyn PostmortemSummaryRule>,
            Rc::new(PostmortemSummaryCodeLengthRule { max_length: 1000 }),
            Rc::new(PostmortemSummaryCountryLengthRule { max_length: 80 }),
            Rc::new(PostmortemSummaryRequiredCountryRule {
                message: "postmortem summary needs a value".to_string(),
            }),
        ],
    }
}
