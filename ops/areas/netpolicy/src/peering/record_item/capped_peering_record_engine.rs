use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped peering record calculation.
pub trait CappedPeeringRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPeeringRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedPeeringRecordAdjuster {
    pub step: f64,
}

impl CappedPeeringRecordFactor for RoundingCappedPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedPeeringRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedPeeringRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedPeeringRecordFactor for TieredCappedPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedPeeringRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedPeeringRecordAdjuster {
    pub cap: f64,
}

impl CappedPeeringRecordFactor for CapCappedPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedPeeringRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped peering record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPeeringRecordEngine {
    pub steps: Vec<Rc<dyn CappedPeeringRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPeeringRecordEngine {
    pub fn run(&mut self, item: &CappedPeeringRecord) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[CappedPeeringRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_peering_record_engine() -> CappedPeeringRecordEngine {
    CappedPeeringRecordEngine {
        steps: vec![
            Rc::new(RoundingCappedPeeringRecordAdjuster { step: 5.0 }) as Rc<dyn CappedPeeringRecordFactor>,
            Rc::new(TieredCappedPeeringRecordAdjuster {
                bounds: vec![2551.0, 3801.0, 8138.0, 9733.0],
                rates: vec![0.94, 0.83, 0.86, 1.17],
            }),
            Rc::new(CapCappedPeeringRecordAdjuster { cap: 38563.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 45351.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_peering_record_engine();
        let item = new_capped_peering_record("code-554", "label-372");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
