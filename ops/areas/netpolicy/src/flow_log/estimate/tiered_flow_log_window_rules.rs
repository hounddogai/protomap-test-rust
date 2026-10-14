use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered flow log window must pass. Check returns its problems, or none.
pub trait TieredFlowLogWindowRule: std::fmt::Debug {
    fn check(&self, item: &TieredFlowLogWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogWindowCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredFlowLogWindowRule for TieredFlowLogWindowCapacityRangeRule {
    fn check(&self, item: &TieredFlowLogWindow) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogWindowTitleLengthRule {
    pub max_length: i64,
}

impl TieredFlowLogWindowRule for TieredFlowLogWindowTitleLengthRule {
    fn check(&self, item: &TieredFlowLogWindow) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogWindowRequiredTitleRule {
    pub message: String,
}

impl TieredFlowLogWindowRule for TieredFlowLogWindowRequiredTitleRule {
    fn check(&self, item: &TieredFlowLogWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered flow log window.
#[derive(Debug, Clone)]
pub struct TieredFlowLogWindowValidator {
    pub rules: Vec<Rc<dyn TieredFlowLogWindowRule>>,
}

impl TieredFlowLogWindowValidator {
    pub fn validate(&self, item: &TieredFlowLogWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredFlowLogWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredFlowLogWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_flow_log_window_validator() -> TieredFlowLogWindowValidator {
    TieredFlowLogWindowValidator {
        rules: vec![
            Rc::new(TieredFlowLogWindowCapacityRangeRule { min: 86400, max: 86400 }) as Rc<dyn TieredFlowLogWindowRule>,
            Rc::new(TieredFlowLogWindowTitleLengthRule { max_length: 16 }),
            Rc::new(TieredFlowLogWindowRequiredTitleRule {
                message: "tiered flow log window needs this field".to_string(),
            }),
        ],
    }
}
