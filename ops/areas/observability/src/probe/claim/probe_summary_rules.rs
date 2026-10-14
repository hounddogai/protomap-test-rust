use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe summary must pass. Check returns its problems, or none.
pub trait ProbeSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ProbeSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSummaryAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ProbeSummaryRule for ProbeSummaryAllowedTitleRule {
    fn check(&self, item: &ProbeSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSummaryBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeSummaryRule for ProbeSummaryBatchSizeRangeRule {
    fn check(&self, item: &ProbeSummary) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSummaryRequiredTitleRule {
    pub message: String,
}

impl ProbeSummaryRule for ProbeSummaryRequiredTitleRule {
    fn check(&self, item: &ProbeSummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe summary.
#[derive(Debug, Clone)]
pub struct ProbeSummaryValidator {
    pub rules: Vec<Rc<dyn ProbeSummaryRule>>,
}

impl ProbeSummaryValidator {
    pub fn validate(&self, item: &ProbeSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_summary_validator() -> ProbeSummaryValidator {
    ProbeSummaryValidator {
        rules: vec![
            Rc::new(ProbeSummaryAllowedTitleRule {
                allowed: vec!["tiered".to_string(), "tiered".to_string(), "manual".to_string()],
            }) as Rc<dyn ProbeSummaryRule>,
            Rc::new(ProbeSummaryBatchSizeRangeRule { min: 1000, max: 3600 }),
            Rc::new(ProbeSummaryRequiredTitleRule {
                message: "probe summary needs an entry".to_string(),
            }),
        ],
    }
}
