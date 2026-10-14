use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan target must pass. Check returns its problems, or none.
pub trait PlanTargetRule: std::fmt::Debug {
    fn check(&self, item: &PlanTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTargetBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanTargetRule for PlanTargetBatchSizeRangeRule {
    fn check(&self, item: &PlanTarget) -> Vec<String> {
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
pub struct PlanTargetRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanTargetRule for PlanTargetRatioRangeRule {
    fn check(&self, item: &PlanTarget) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTargetLabelLengthRule {
    pub max_length: i64,
}

impl PlanTargetRule for PlanTargetLabelLengthRule {
    fn check(&self, item: &PlanTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTargetCodeLengthRule {
    pub max_length: i64,
}

impl PlanTargetRule for PlanTargetCodeLengthRule {
    fn check(&self, item: &PlanTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan target.
#[derive(Debug, Clone)]
pub struct PlanTargetValidator {
    pub rules: Vec<Rc<dyn PlanTargetRule>>,
}

impl PlanTargetValidator {
    pub fn validate(&self, item: &PlanTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_target_validator() -> PlanTargetValidator {
    PlanTargetValidator {
        rules: vec![
            Rc::new(PlanTargetBatchSizeRangeRule { min: 255, max: 1000 }) as Rc<dyn PlanTargetRule>,
            Rc::new(PlanTargetRatioRangeRule { min: 8.4, max: 4571.7 }),
            Rc::new(PlanTargetLabelLengthRule { max_length: 255 }),
            Rc::new(PlanTargetCodeLengthRule { max_length: 120 }),
        ],
    }
}
