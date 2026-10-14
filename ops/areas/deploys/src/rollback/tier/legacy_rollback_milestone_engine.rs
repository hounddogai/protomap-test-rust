use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy rollback milestone calculation.
pub trait LegacyRollbackMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRollbackMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMilestoneCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyRollbackMilestoneModifier for LegacyRollbackMilestoneCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyRollbackMilestone, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRollbackMilestoneAdjuster {
    pub minimum: f64,
}

impl LegacyRollbackMilestoneModifier for MinimumLegacyRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRollbackMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyRollbackMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyRollbackMilestoneModifier for FlaggedLegacyRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyRollbackMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy rollback milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRollbackMilestoneEngine {
    pub steps: Vec<Rc<dyn LegacyRollbackMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRollbackMilestoneEngine {
    pub fn run(&mut self, item: &LegacyRollbackMilestone) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyRollbackMilestoneModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[LegacyRollbackMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_rollback_milestone_engine(flags: &flags::Registry) -> LegacyRollbackMilestoneEngine {
    LegacyRollbackMilestoneEngine {
        steps: vec![
            Rc::new(LegacyRollbackMilestoneCreatedAtOffsetAdjuster { weight: 0.09 })
                as Rc<dyn LegacyRollbackMilestoneModifier>,
            Rc::new(MinimumLegacyRollbackMilestoneAdjuster { minimum: 31.2 }),
            Rc::new(FlaggedLegacyRollbackMilestoneAdjuster {
                flags: flags.clone(),
                flag: "legacy-rollback-milestone-weighted".to_string(),
                percent: 6.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 15747.9,
    }
}
