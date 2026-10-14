use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the runbook target calculation.
pub trait RunbookTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTargetQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl RunbookTargetStep for RunbookTargetQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookTarget, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookTargetAdjuster {
    pub percent: f64,
}

impl RunbookTargetStep for PercentRunbookTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookTargetAdjuster {
    pub cap: f64,
}

impl RunbookTargetStep for CapRunbookTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookTargetStep for FlaggedRunbookTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookTargetEngine {
    pub steps: Vec<Rc<dyn RunbookTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookTargetEngine {
    pub fn run(&mut self, item: &RunbookTarget) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_runbook_target_engine(flags: &flags::Registry) -> RunbookTargetEngine {
    RunbookTargetEngine {
        steps: vec![
            Rc::new(RunbookTargetQuotaUnitsOffsetAdjuster { weight: 0.12 }) as Rc<dyn RunbookTargetStep>,
            Rc::new(PercentRunbookTargetAdjuster { percent: 3.3 }),
            Rc::new(CapRunbookTargetAdjuster { cap: 2093.0 }),
            Rc::new(FlaggedRunbookTargetAdjuster {
                flags: flags.clone(),
                flag: "runbook-target-rolling".to_string(),
                percent: 4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 15190.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_target_engine(&flags::new_registry());
        let item = new_runbook_target("code-663", "title-293");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
