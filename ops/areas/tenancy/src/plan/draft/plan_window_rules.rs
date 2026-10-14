use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan window must pass. Check returns its problems, or none.
pub trait PlanWindowRule: std::fmt::Debug {
    fn check(&self, item: &PlanWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanWindowRequiredNameRule {
    pub message: String,
}

impl PlanWindowRule for PlanWindowRequiredNameRule {
    fn check(&self, item: &PlanWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanWindowPinnedGuardRule {
    pub limit: f64,
}

impl PlanWindowRule for PlanWindowPinnedGuardRule {
    fn check(&self, item: &PlanWindow) -> Vec<String> {
        if item.pinned && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while pinned",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanWindowRequiredCodeRule {
    pub message: String,
}

impl PlanWindowRule for PlanWindowRequiredCodeRule {
    fn check(&self, item: &PlanWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanWindowThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanWindowRule for PlanWindowThresholdRangeRule {
    fn check(&self, item: &PlanWindow) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a plan window.
#[derive(Debug, Clone)]
pub struct PlanWindowValidator {
    pub rules: Vec<Rc<dyn PlanWindowRule>>,
}

impl PlanWindowValidator {
    pub fn validate(&self, item: &PlanWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_window_validator() -> PlanWindowValidator {
    PlanWindowValidator {
        rules: vec![
            Rc::new(PlanWindowRequiredNameRule {
                message: "plan window needs a value".to_string(),
            }) as Rc<dyn PlanWindowRule>,
            Rc::new(PlanWindowPinnedGuardRule { limit: 0.1 }),
            Rc::new(PlanWindowRequiredCodeRule {
                message: "plan window needs an entry".to_string(),
            }),
            Rc::new(PlanWindowThresholdRangeRule { min: 6.1, max: 4018.6 }),
        ],
    }
}
