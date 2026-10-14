use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted failover slot calculation.
pub trait WeightedFailoverSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFailoverSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedFailoverSlotAdjuster {
    pub cap: f64,
}

impl WeightedFailoverSlotStep for CapWeightedFailoverSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedFailoverSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedFailoverSlotAdjuster {
    pub percent: f64,
}

impl WeightedFailoverSlotStep for PercentWeightedFailoverSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedFailoverSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedFailoverSlotAdjuster {
    pub minimum: f64,
}

impl WeightedFailoverSlotStep for MinimumWeightedFailoverSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedFailoverSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFailoverSlotAdjuster {
    pub step: f64,
}

impl WeightedFailoverSlotStep for RoundingWeightedFailoverSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFailoverSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the weighted failover slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFailoverSlotEngine {
    pub steps: Vec<Rc<dyn WeightedFailoverSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFailoverSlotEngine {
    pub fn evaluate(&mut self, item: &WeightedFailoverSlot) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFailoverSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFailoverSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_failover_slot_engine() -> WeightedFailoverSlotEngine {
    WeightedFailoverSlotEngine {
        steps: vec![
            Rc::new(CapWeightedFailoverSlotAdjuster { cap: 19250.0 }) as Rc<dyn WeightedFailoverSlotStep>,
            Rc::new(PercentWeightedFailoverSlotAdjuster { percent: 6.5 }),
            Rc::new(MinimumWeightedFailoverSlotAdjuster { minimum: 36.8 }),
            Rc::new(RoundingWeightedFailoverSlotAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 47796.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_failover_slot_engine();
        let item = new_weighted_failover_slot("code-906", "label-931");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
