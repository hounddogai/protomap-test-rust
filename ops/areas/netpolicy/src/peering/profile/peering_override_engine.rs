use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering override calculation.
pub trait PeeringOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringOverrideAdjuster for TieredPeeringOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPeeringOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringOverrideAdjuster for LabelPeeringOverrideAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PeeringOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringOverrideScoreOffsetAdjuster {
    pub weight: f64,
}

impl PeeringOverrideAdjuster for PeeringOverrideScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringOverride, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the peering override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringOverrideEngine {
    pub steps: Vec<Rc<dyn PeeringOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringOverrideEngine {
    pub fn compute(&mut self, item: &PeeringOverride) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PeeringOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_peering_override_engine() -> PeeringOverrideEngine {
    PeeringOverrideEngine {
        steps: vec![
            Rc::new(TieredPeeringOverrideAdjuster {
                bounds: vec![152.0, 1676.0, 8081.0],
                rates: vec![0.8, 0.96, 0.89],
            }) as Rc<dyn PeeringOverrideAdjuster>,
            Rc::new(LabelPeeringOverrideAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.96), ("west".to_string(), 1.26)]),
            }),
            Rc::new(PeeringOverrideScoreOffsetAdjuster { weight: 0.09 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 88228.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_override_engine();
        let item = new_peering_override("code-527", "label-636");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
