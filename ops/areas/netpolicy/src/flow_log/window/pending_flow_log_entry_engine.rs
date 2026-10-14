use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending flow log entry calculation.
pub trait PendingFlowLogEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingFlowLogEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEntryFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl PendingFlowLogEntryModifier for PendingFlowLogEntryFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &PendingFlowLogEntry, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingFlowLogEntryAdjuster {
    pub cap: f64,
}

impl PendingFlowLogEntryModifier for CapPendingFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingFlowLogEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingFlowLogEntryAdjuster {
    pub step: f64,
}

impl PendingFlowLogEntryModifier for RoundingPendingFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingFlowLogEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingFlowLogEntryAdjuster {
    pub percent: f64,
}

impl PendingFlowLogEntryModifier for PercentPendingFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingFlowLogEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingFlowLogEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingFlowLogEntryModifier for TieredPendingFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingFlowLogEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending flow log entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingFlowLogEntryEngine {
    pub steps: Vec<Rc<dyn PendingFlowLogEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingFlowLogEntryEngine {
    pub fn price(&mut self, item: &PendingFlowLogEntry) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingFlowLogEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingFlowLogEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_flow_log_entry_engine() -> PendingFlowLogEntryEngine {
    PendingFlowLogEntryEngine {
        steps: vec![
            Rc::new(PendingFlowLogEntryFloorValueOffsetAdjuster { weight: 0.35 })
                as Rc<dyn PendingFlowLogEntryModifier>,
            Rc::new(CapPendingFlowLogEntryAdjuster { cap: 43141.0 }),
            Rc::new(RoundingPendingFlowLogEntryAdjuster { step: 0.5 }),
            Rc::new(PercentPendingFlowLogEntryAdjuster { percent: -8.0 }),
            Rc::new(TieredPendingFlowLogEntryAdjuster {
                bounds: vec![640.0, 7054.0],
                rates: vec![0.83, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 73286.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_flow_log_entry_engine();
        let item = new_pending_flow_log_entry("code-377", "label-518");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
