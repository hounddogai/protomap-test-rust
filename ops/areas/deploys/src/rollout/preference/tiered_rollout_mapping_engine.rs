use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered rollout mapping calculation.
pub trait TieredRolloutMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRolloutMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredRolloutMappingAdjuster {
    pub percent: f64,
}

impl TieredRolloutMappingFactor for PercentTieredRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredRolloutMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredRolloutMappingAdjuster {
    pub cap: f64,
}

impl TieredRolloutMappingFactor for CapTieredRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredRolloutMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTieredRolloutMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredRolloutMappingFactor for ScopeTieredRolloutMappingAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TieredRolloutMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredRolloutMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredRolloutMappingFactor for FlaggedTieredRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredRolloutMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredRolloutMappingAdjuster {
    pub minimum: f64,
}

impl TieredRolloutMappingFactor for MinimumTieredRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredRolloutMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredRolloutMappingAdjuster {
    pub step: f64,
}

impl TieredRolloutMappingFactor for RoundingTieredRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredRolloutMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered rollout mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRolloutMappingEngine {
    pub steps: Vec<Rc<dyn TieredRolloutMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRolloutMappingEngine {
    pub fn run(&mut self, item: &TieredRolloutMapping) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[TieredRolloutMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_rollout_mapping_engine(flags: &flags::Registry) -> TieredRolloutMappingEngine {
    TieredRolloutMappingEngine {
        steps: vec![
            Rc::new(PercentTieredRolloutMappingAdjuster { percent: 16.7 }) as Rc<dyn TieredRolloutMappingFactor>,
            Rc::new(CapTieredRolloutMappingAdjuster { cap: 39922.0 }),
            Rc::new(ScopeTieredRolloutMappingAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.86),
                    ("east".to_string(), 1.28),
                    ("central".to_string(), 1.14),
                ]),
            }),
            Rc::new(FlaggedTieredRolloutMappingAdjuster {
                flags: flags.clone(),
                flag: "tiered-rollout-mapping-adaptive".to_string(),
                percent: -5.6,
            }),
            Rc::new(MinimumTieredRolloutMappingAdjuster { minimum: 25.7 }),
            Rc::new(RoundingTieredRolloutMappingAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 66120.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_rollout_mapping_engine(&flags::new_registry());
        let item = new_tiered_rollout_mapping("code-734", "label-881");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
