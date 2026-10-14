use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal flow log assignment must pass. Check returns its problems, or none.
pub trait SeasonalFlowLogAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFlowLogAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFlowLogAssignmentTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalFlowLogAssignmentRule for SeasonalFlowLogAssignmentTitleLengthRule {
    fn check(&self, item: &SeasonalFlowLogAssignment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFlowLogAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalFlowLogAssignmentRule for SeasonalFlowLogAssignmentCodeLengthRule {
    fn check(&self, item: &SeasonalFlowLogAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFlowLogAssignmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalFlowLogAssignmentRule for SeasonalFlowLogAssignmentQuantityRangeRule {
    fn check(&self, item: &SeasonalFlowLogAssignment) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFlowLogAssignmentEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalFlowLogAssignmentRule for SeasonalFlowLogAssignmentEndsAtRangeRule {
    fn check(&self, item: &SeasonalFlowLogAssignment) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFlowLogAssignmentThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalFlowLogAssignmentRule for SeasonalFlowLogAssignmentThresholdRangeRule {
    fn check(&self, item: &SeasonalFlowLogAssignment) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFlowLogAssignmentRequiredCodeRule {
    pub message: String,
}

impl SeasonalFlowLogAssignmentRule for SeasonalFlowLogAssignmentRequiredCodeRule {
    fn check(&self, item: &SeasonalFlowLogAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal flow log assignment.
#[derive(Debug, Clone)]
pub struct SeasonalFlowLogAssignmentValidator {
    pub rules: Vec<Rc<dyn SeasonalFlowLogAssignmentRule>>,
}

impl SeasonalFlowLogAssignmentValidator {
    pub fn validate(&self, item: &SeasonalFlowLogAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalFlowLogAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFlowLogAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_flow_log_assignment_validator() -> SeasonalFlowLogAssignmentValidator {
    SeasonalFlowLogAssignmentValidator {
        rules: vec![
            Rc::new(SeasonalFlowLogAssignmentTitleLengthRule { max_length: 120 })
                as Rc<dyn SeasonalFlowLogAssignmentRule>,
            Rc::new(SeasonalFlowLogAssignmentCodeLengthRule { max_length: 86400 }),
            Rc::new(SeasonalFlowLogAssignmentQuantityRangeRule { min: 32, max: 1000 }),
            Rc::new(SeasonalFlowLogAssignmentEndsAtRangeRule { min: 16, max: 32 }),
            Rc::new(SeasonalFlowLogAssignmentThresholdRangeRule { min: 5.2, max: 619.5 }),
            Rc::new(SeasonalFlowLogAssignmentRequiredCodeRule {
                message: "seasonal flow log assignment needs an entry".to_string(),
            }),
        ],
    }
}
