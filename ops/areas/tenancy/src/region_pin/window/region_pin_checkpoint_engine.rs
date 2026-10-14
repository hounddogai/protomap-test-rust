use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin checkpoint calculation.
pub trait RegionPinCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinCheckpointAdjuster {
    pub minimum: f64,
}

impl RegionPinCheckpointModifier for MinimumRegionPinCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinCheckpointModifier for FlaggedRegionPinCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinCheckpointAdjuster {
    pub step: f64,
}

impl RegionPinCheckpointModifier for RoundingRegionPinCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionPinCheckpointAdjuster {
    pub percent: f64,
}

impl RegionPinCheckpointModifier for PercentRegionPinCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionPinCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRegionPinCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinCheckpointModifier for CategoryRegionPinCheckpointAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RegionPinCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinCheckpointAdjuster {
    pub cap: f64,
}

impl RegionPinCheckpointModifier for CapRegionPinCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the region pin checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinCheckpointEngine {
    pub steps: Vec<Rc<dyn RegionPinCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinCheckpointEngine {
    pub fn compute(&mut self, item: &RegionPinCheckpoint) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinCheckpointModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RegionPinCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_region_pin_checkpoint_engine(flags: &flags::Registry) -> RegionPinCheckpointEngine {
    RegionPinCheckpointEngine {
        steps: vec![
            Rc::new(MinimumRegionPinCheckpointAdjuster { minimum: 11.7 }) as Rc<dyn RegionPinCheckpointModifier>,
            Rc::new(FlaggedRegionPinCheckpointAdjuster {
                flags: flags.clone(),
                flag: "region-pin-checkpoint-regional".to_string(),
                percent: -8.0,
            }),
            Rc::new(RoundingRegionPinCheckpointAdjuster { step: 0.05 }),
            Rc::new(PercentRegionPinCheckpointAdjuster { percent: -20.8 }),
            Rc::new(CategoryRegionPinCheckpointAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.85), ("west".to_string(), 0.88)]),
            }),
            Rc::new(CapRegionPinCheckpointAdjuster { cap: 39293.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 18982.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_checkpoint_engine(&flags::new_registry());
        let item = new_region_pin_checkpoint("code-323", "title-550");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
