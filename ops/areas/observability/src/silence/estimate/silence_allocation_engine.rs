use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the silence allocation calculation.
pub trait SilenceAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceAllocationAdjuster {
    pub percent: f64,
}

impl SilenceAllocationStep for PercentSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAllocationAmountOffsetAdjuster {
    pub weight: f64,
}

impl SilenceAllocationStep for SilenceAllocationAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceAllocation, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceAllocationAdjuster {
    pub step: f64,
}

impl SilenceAllocationStep for RoundingSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the silence allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceAllocationEngine {
    pub steps: Vec<Rc<dyn SilenceAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceAllocationEngine {
    pub fn run(&mut self, item: &SilenceAllocation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceAllocationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_silence_allocation_engine() -> SilenceAllocationEngine {
    SilenceAllocationEngine {
        steps: vec![
            Rc::new(PercentSilenceAllocationAdjuster { percent: -4.8 }) as Rc<dyn SilenceAllocationStep>,
            Rc::new(SilenceAllocationAmountOffsetAdjuster { weight: 1.86 }),
            Rc::new(RoundingSilenceAllocationAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 67030.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_allocation_engine();
        let item = new_silence_allocation("code-608", "name-810");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
