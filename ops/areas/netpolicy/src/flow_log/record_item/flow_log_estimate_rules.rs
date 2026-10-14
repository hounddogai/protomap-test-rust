use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log estimate must pass. Check returns its problems, or none.
pub trait FlowLogEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEstimateLabelLengthRule {
    pub max_length: i64,
}

impl FlowLogEstimateRule for FlowLogEstimateLabelLengthRule {
    fn check(&self, item: &FlowLogEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEstimateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogEstimateRule for FlowLogEstimateScoreRangeRule {
    fn check(&self, item: &FlowLogEstimate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEstimateVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogEstimateRule for FlowLogEstimateVersionRangeRule {
    fn check(&self, item: &FlowLogEstimate) -> Vec<String> {
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

/// Runs every rule of a flow log estimate.
#[derive(Debug, Clone)]
pub struct FlowLogEstimateValidator {
    pub rules: Vec<Rc<dyn FlowLogEstimateRule>>,
}

impl FlowLogEstimateValidator {
    pub fn validate(&self, item: &FlowLogEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_estimate_validator() -> FlowLogEstimateValidator {
    FlowLogEstimateValidator {
        rules: vec![
            Rc::new(FlowLogEstimateLabelLengthRule { max_length: 1000 }) as Rc<dyn FlowLogEstimateRule>,
            Rc::new(FlowLogEstimateScoreRangeRule { min: 4.7, max: 1950.0 }),
            Rc::new(FlowLogEstimateVersionRangeRule { min: 1000, max: 1000 }),
        ],
    }
}
