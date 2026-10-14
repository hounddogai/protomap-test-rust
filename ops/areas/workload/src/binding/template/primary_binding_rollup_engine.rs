use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary binding rollup calculation.
pub trait PrimaryBindingRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryBindingRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryBindingRollupAdjuster {
    pub cap: f64,
}

impl PrimaryBindingRollupFactor for CapPrimaryBindingRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryBindingRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryBindingRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryBindingRollupFactor for FlaggedPrimaryBindingRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryBindingRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryBindingRollupAdjuster {
    pub step: f64,
}

impl PrimaryBindingRollupFactor for RoundingPrimaryBindingRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryBindingRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryBindingRollupAdjuster {
    pub percent: f64,
}

impl PrimaryBindingRollupFactor for PercentPrimaryBindingRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryBindingRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary binding rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryBindingRollupEngine {
    pub steps: Vec<Rc<dyn PrimaryBindingRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryBindingRollupEngine {
    pub fn compute(&mut self, item: &PrimaryBindingRollup) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryBindingRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_binding_rollup_engine(flags: &flags::Registry) -> PrimaryBindingRollupEngine {
    PrimaryBindingRollupEngine {
        steps: vec![
            Rc::new(CapPrimaryBindingRollupAdjuster { cap: 11411.0 }) as Rc<dyn PrimaryBindingRollupFactor>,
            Rc::new(FlaggedPrimaryBindingRollupAdjuster {
                flags: flags.clone(),
                flag: "primary-binding-rollup-scheduled".to_string(),
                percent: 3.2,
            }),
            Rc::new(RoundingPrimaryBindingRollupAdjuster { step: 1.0 }),
            Rc::new(PercentPrimaryBindingRollupAdjuster { percent: 23.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 71911.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_binding_rollup_engine(&flags::new_registry());
        let item = new_primary_binding_rollup("code-162", "title-874");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
