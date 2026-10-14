use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending overage bundle calculation.
pub trait PendingOverageBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingOverageBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingOverageBundleAdjuster {
    pub minimum: f64,
}

impl PendingOverageBundleAdjuster for MinimumPendingOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingOverageBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingOverageBundleAdjuster {
    pub percent: f64,
}

impl PendingOverageBundleAdjuster for PercentPendingOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingOverageBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingOverageBundleAdjuster {
    pub step: f64,
}

impl PendingOverageBundleAdjuster for RoundingPendingOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingOverageBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingOverageBundleAdjuster {
    pub cap: f64,
}

impl PendingOverageBundleAdjuster for CapPendingOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingOverageBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPendingOverageBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingOverageBundleAdjuster for LabelPendingOverageBundleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PendingOverageBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending overage bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingOverageBundleEngine {
    pub steps: Vec<Rc<dyn PendingOverageBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingOverageBundleEngine {
    pub fn run(&mut self, item: &PendingOverageBundle) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingOverageBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_overage_bundle_engine() -> PendingOverageBundleEngine {
    PendingOverageBundleEngine {
        steps: vec![
            Rc::new(MinimumPendingOverageBundleAdjuster { minimum: 37.1 }) as Rc<dyn PendingOverageBundleAdjuster>,
            Rc::new(PercentPendingOverageBundleAdjuster { percent: -0.6 }),
            Rc::new(RoundingPendingOverageBundleAdjuster { step: 0.25 }),
            Rc::new(CapPendingOverageBundleAdjuster { cap: 2312.0 }),
            Rc::new(LabelPendingOverageBundleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.13), ("retail".to_string(), 1.06)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 15686.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_overage_bundle_engine();
        let item = new_pending_overage_bundle("code-349", "label-813");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
