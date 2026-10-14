use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering variance calculation.
pub trait PeeringVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringVarianceWeightOffsetAdjuster {
    pub weight: f64,
}

impl PeeringVarianceModifier for PeeringVarianceWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringVariance, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringVarianceAdjuster {
    pub step: f64,
}

impl PeeringVarianceModifier for RoundingPeeringVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringVarianceAdjuster {
    pub percent: f64,
}

impl PeeringVarianceModifier for PercentPeeringVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringVarianceAdjuster {
    pub minimum: f64,
}

impl PeeringVarianceModifier for MinimumPeeringVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringVarianceAdjuster {
    pub cap: f64,
}

impl PeeringVarianceModifier for CapPeeringVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the peering variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringVarianceEngine {
    pub steps: Vec<Rc<dyn PeeringVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringVarianceEngine {
    pub fn evaluate(&mut self, item: &PeeringVariance) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[PeeringVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_variance_engine() -> PeeringVarianceEngine {
    PeeringVarianceEngine {
        steps: vec![
            Rc::new(PeeringVarianceWeightOffsetAdjuster { weight: 1.67 }) as Rc<dyn PeeringVarianceModifier>,
            Rc::new(RoundingPeeringVarianceAdjuster { step: 5.0 }),
            Rc::new(PercentPeeringVarianceAdjuster { percent: -5.6 }),
            Rc::new(MinimumPeeringVarianceAdjuster { minimum: 5.0 }),
            Rc::new(CapPeeringVarianceAdjuster { cap: 40872.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 15285.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_variance_engine();
        let item = new_peering_variance("code-906", "label-233");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
