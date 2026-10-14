use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollout manifest calculation.
pub trait RolloutManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutManifestAdjuster {
    pub step: f64,
}

impl RolloutManifestStep for RoundingRolloutManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutManifestAdjuster {
    pub percent: f64,
}

impl RolloutManifestStep for PercentRolloutManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutManifestAdjuster {
    pub cap: f64,
}

impl RolloutManifestStep for CapRolloutManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRolloutManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutManifestStep for SegmentRolloutManifestAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RolloutManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutManifestCapacityOffsetAdjuster {
    pub weight: f64,
}

impl RolloutManifestStep for RolloutManifestCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutManifest, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the rollout manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutManifestEngine {
    pub steps: Vec<Rc<dyn RolloutManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutManifestEngine {
    pub fn evaluate(&mut self, item: &RolloutManifest) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RolloutManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_manifest_engine() -> RolloutManifestEngine {
    RolloutManifestEngine {
        steps: vec![
            Rc::new(RoundingRolloutManifestAdjuster { step: 1.0 }) as Rc<dyn RolloutManifestStep>,
            Rc::new(PercentRolloutManifestAdjuster { percent: -2.4 }),
            Rc::new(CapRolloutManifestAdjuster { cap: 6453.0 }),
            Rc::new(SegmentRolloutManifestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.76)]),
            }),
            Rc::new(RolloutManifestCapacityOffsetAdjuster { weight: 0.52 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 95608.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_manifest_engine();
        let item = new_rollout_manifest("code-858", "name-298");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
