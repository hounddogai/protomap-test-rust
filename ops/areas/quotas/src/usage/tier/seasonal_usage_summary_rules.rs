use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal usage summary must pass. Check returns its problems, or none.
pub trait SeasonalUsageSummaryRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalUsageSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageSummaryAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl SeasonalUsageSummaryRule for SeasonalUsageSummaryAllowedCategoryRule {
    fn check(&self, item: &SeasonalUsageSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is out of range", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageSummaryRequiredSegmentRule {
    pub message: String,
}

impl SeasonalUsageSummaryRule for SeasonalUsageSummaryRequiredSegmentRule {
    fn check(&self, item: &SeasonalUsageSummary) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageSummaryCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalUsageSummaryRule for SeasonalUsageSummaryCodeLengthRule {
    fn check(&self, item: &SeasonalUsageSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal usage summary.
#[derive(Debug, Clone)]
pub struct SeasonalUsageSummaryValidator {
    pub rules: Vec<Rc<dyn SeasonalUsageSummaryRule>>,
}

impl SeasonalUsageSummaryValidator {
    pub fn validate(&self, item: &SeasonalUsageSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalUsageSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalUsageSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_usage_summary_validator() -> SeasonalUsageSummaryValidator {
    SeasonalUsageSummaryValidator {
        rules: vec![
            Rc::new(SeasonalUsageSummaryAllowedCategoryRule {
                allowed: vec!["fallback".to_string()],
            }) as Rc<dyn SeasonalUsageSummaryRule>,
            Rc::new(SeasonalUsageSummaryRequiredSegmentRule {
                message: "seasonal usage summary needs this field".to_string(),
            }),
            Rc::new(SeasonalUsageSummaryCodeLengthRule { max_length: 80 }),
        ],
    }
}
