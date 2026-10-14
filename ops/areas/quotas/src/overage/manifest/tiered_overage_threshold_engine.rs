use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered overage threshold calculation.
pub trait TieredOverageThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredOverageThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredOverageThresholdAdjuster {
    pub percent: f64,
}

impl TieredOverageThresholdModifier for PercentTieredOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredOverageThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredOverageThresholdAdjuster {
    pub minimum: f64,
}

impl TieredOverageThresholdModifier for MinimumTieredOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredOverageThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredOverageThresholdAdjuster {
    pub cap: f64,
}

impl TieredOverageThresholdModifier for CapTieredOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredOverageThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredOverageThresholdAdjuster {
    pub step: f64,
}

impl TieredOverageThresholdModifier for RoundingTieredOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredOverageThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredOverageThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredOverageThresholdModifier for FlaggedTieredOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredOverageThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered overage threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredOverageThresholdEngine {
    pub steps: Vec<Rc<dyn TieredOverageThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredOverageThresholdEngine {
    pub fn price(&mut self, item: &TieredOverageThreshold) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredOverageThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_overage_threshold_engine(flags: &flags::Registry) -> TieredOverageThresholdEngine {
    TieredOverageThresholdEngine {
        steps: vec![
            Rc::new(PercentTieredOverageThresholdAdjuster { percent: -18.4 }) as Rc<dyn TieredOverageThresholdModifier>,
            Rc::new(MinimumTieredOverageThresholdAdjuster { minimum: 48.5 }),
            Rc::new(CapTieredOverageThresholdAdjuster { cap: 23718.0 }),
            Rc::new(RoundingTieredOverageThresholdAdjuster { step: 0.5 }),
            Rc::new(FlaggedTieredOverageThresholdAdjuster {
                flags: flags.clone(),
                flag: "tiered-overage-threshold-strict".to_string(),
                percent: 3.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 47515.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_overage_threshold_engine(&flags::new_registry());
        let item = new_tiered_overage_threshold("code-46", "name-195");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
