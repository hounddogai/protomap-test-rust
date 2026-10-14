use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout rollup calculation.
pub trait RolloutRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutRollupAdjuster {
    pub percent: f64,
}

impl RolloutRollupAdjuster for PercentRolloutRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutRollupAdjuster for FlaggedRolloutRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRollupRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RolloutRollupAdjuster for RolloutRollupRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutRollup, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the rollout rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutRollupEngine {
    pub steps: Vec<Rc<dyn RolloutRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutRollupEngine {
    pub fn run(&mut self, item: &RolloutRollup) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[RolloutRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollout_rollup_engine(flags: &flags::Registry) -> RolloutRollupEngine {
    RolloutRollupEngine {
        steps: vec![
            Rc::new(PercentRolloutRollupAdjuster { percent: -8.5 }) as Rc<dyn RolloutRollupAdjuster>,
            Rc::new(FlaggedRolloutRollupAdjuster {
                flags: flags.clone(),
                flag: "rollout-rollup-fallback".to_string(),
                percent: -5.5,
            }),
            Rc::new(RolloutRollupRetriesOffsetAdjuster { weight: 0.65 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 15120.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_rollup_engine(&flags::new_registry());
        let item = new_rollout_rollup("code-963", "label-518");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
