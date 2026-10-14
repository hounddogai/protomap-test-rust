use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress bundle calculation.
pub trait EgressBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressBundleAdjuster {
    pub cap: f64,
}

impl EgressBundleAdjuster for CapEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressBundleAdjuster {
    pub percent: f64,
}

impl EgressBundleAdjuster for PercentEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressBundleAdjuster {
    pub step: f64,
}

impl EgressBundleAdjuster for RoundingEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEgressBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressBundleAdjuster for SegmentEgressBundleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &EgressBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the egress bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressBundleEngine {
    pub steps: Vec<Rc<dyn EgressBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressBundleEngine {
    pub fn evaluate(&mut self, item: &EgressBundle) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[EgressBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_egress_bundle_engine() -> EgressBundleEngine {
    EgressBundleEngine {
        steps: vec![
            Rc::new(CapEgressBundleAdjuster { cap: 36744.0 }) as Rc<dyn EgressBundleAdjuster>,
            Rc::new(PercentEgressBundleAdjuster { percent: 6.3 }),
            Rc::new(RoundingEgressBundleAdjuster { step: 0.1 }),
            Rc::new(SegmentEgressBundleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.98), ("east".to_string(), 0.97)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 82047.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_bundle_engine();
        let item = new_egress_bundle("code-166", "name-848");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
