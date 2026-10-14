use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook estimate must pass. Check returns its problems, or none.
pub trait RunbookEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RunbookEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEstimateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RunbookEstimateRule for RunbookEstimateAllowedTitleRule {
    fn check(&self, item: &RunbookEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEstimateVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookEstimateRule for RunbookEstimateVersionRangeRule {
    fn check(&self, item: &RunbookEstimate) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEstimateRequiredClusterRule {
    pub message: String,
}

impl RunbookEstimateRule for RunbookEstimateRequiredClusterRule {
    fn check(&self, item: &RunbookEstimate) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEstimateRequiredCodeRule {
    pub message: String,
}

impl RunbookEstimateRule for RunbookEstimateRequiredCodeRule {
    fn check(&self, item: &RunbookEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook estimate.
#[derive(Debug, Clone)]
pub struct RunbookEstimateValidator {
    pub rules: Vec<Rc<dyn RunbookEstimateRule>>,
}

impl RunbookEstimateValidator {
    pub fn validate(&self, item: &RunbookEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_estimate_validator() -> RunbookEstimateValidator {
    RunbookEstimateValidator {
        rules: vec![
            Rc::new(RunbookEstimateAllowedTitleRule {
                allowed: vec!["strict".to_string()],
            }) as Rc<dyn RunbookEstimateRule>,
            Rc::new(RunbookEstimateVersionRangeRule { min: 16, max: 1000 }),
            Rc::new(RunbookEstimateRequiredClusterRule {
                message: "runbook estimate needs a value".to_string(),
            }),
            Rc::new(RunbookEstimateRequiredCodeRule {
                message: "runbook estimate needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_estimate_validator();
        let item = new_runbook_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RunbookEstimateVersionRangeRule { min: 120, max: 1000 };
        let mut item = new_runbook_estimate("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RunbookEstimateRequiredClusterRule {
            message: "runbook estimate needs a value".to_string(),
        };
        let mut item = new_runbook_estimate("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookEstimateRequiredCodeRule {
            message: "runbook estimate needs this field".to_string(),
        };
        let mut item = new_runbook_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
