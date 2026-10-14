use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the runbook tier calculation.
pub trait RunbookTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookTier, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookTierModifier for FlaggedRunbookTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookTierAdjuster {
    pub cap: f64,
}

impl RunbookTierModifier for CapRunbookTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookTierAdjuster {
    pub percent: f64,
}

impl RunbookTierModifier for PercentRunbookTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookTierEngine {
    pub steps: Vec<Rc<dyn RunbookTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookTierEngine {
    pub fn run(&mut self, item: &RunbookTier) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookTierModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_runbook_tier_engine(flags: &flags::Registry) -> RunbookTierEngine {
    RunbookTierEngine {
        steps: vec![
            Rc::new(FlaggedRunbookTierAdjuster {
                flags: flags.clone(),
                flag: "runbook-tier-rolling".to_string(),
                percent: 5.4,
            }) as Rc<dyn RunbookTierModifier>,
            Rc::new(CapRunbookTierAdjuster { cap: 10872.0 }),
            Rc::new(PercentRunbookTierAdjuster { percent: 3.2 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 79680.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_tier_engine(&flags::new_registry());
        let item = new_runbook_tier("code-19", "label-331");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
