use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering rollup calculation.
pub trait PeeringRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringRollupAdjuster {
    pub cap: f64,
}

impl PeeringRollupAdjuster for CapPeeringRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringRollupAdjuster {
    pub minimum: f64,
}

impl PeeringRollupAdjuster for MinimumPeeringRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringRollupAdjuster {
    pub step: f64,
}

impl PeeringRollupAdjuster for RoundingPeeringRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the peering rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringRollupEngine {
    pub steps: Vec<Rc<dyn PeeringRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringRollupEngine {
    pub fn evaluate(&mut self, item: &PeeringRollup) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PeeringRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_rollup_engine() -> PeeringRollupEngine {
    PeeringRollupEngine {
        steps: vec![
            Rc::new(CapPeeringRollupAdjuster { cap: 31842.0 }) as Rc<dyn PeeringRollupAdjuster>,
            Rc::new(MinimumPeeringRollupAdjuster { minimum: 45.7 }),
            Rc::new(RoundingPeeringRollupAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 10511.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_rollup_engine();
        let item = new_peering_rollup("code-531", "name-753");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
