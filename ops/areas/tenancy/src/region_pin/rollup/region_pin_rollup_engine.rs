use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin rollup calculation.
pub trait RegionPinRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRollupQuantityOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinRollupAdjuster for RegionPinRollupQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinRollup, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinRollupAdjuster {
    pub cap: f64,
}

impl RegionPinRollupAdjuster for CapRegionPinRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinRollupAdjuster for TieredRegionPinRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionPinRollupAdjuster {
    pub percent: f64,
}

impl RegionPinRollupAdjuster for PercentRegionPinRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionPinRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinRollupAdjuster {
    pub step: f64,
}

impl RegionPinRollupAdjuster for RoundingRegionPinRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinRollupAdjuster for FlaggedRegionPinRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the region pin rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinRollupEngine {
    pub steps: Vec<Rc<dyn RegionPinRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinRollupEngine {
    pub fn compute(&mut self, item: &RegionPinRollup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinRollupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_region_pin_rollup_engine(flags: &flags::Registry) -> RegionPinRollupEngine {
    RegionPinRollupEngine {
        steps: vec![
            Rc::new(RegionPinRollupQuantityOffsetAdjuster { weight: 1.47 }) as Rc<dyn RegionPinRollupAdjuster>,
            Rc::new(CapRegionPinRollupAdjuster { cap: 35979.0 }),
            Rc::new(TieredRegionPinRollupAdjuster {
                bounds: vec![4745.0, 5680.0],
                rates: vec![0.85, 1.08],
            }),
            Rc::new(PercentRegionPinRollupAdjuster { percent: 17.5 }),
            Rc::new(RoundingRegionPinRollupAdjuster { step: 0.5 }),
            Rc::new(FlaggedRegionPinRollupAdjuster {
                flags: flags.clone(),
                flag: "region-pin-rollup-default".to_string(),
                percent: -0.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 81761.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_rollup_engine(&flags::new_registry());
        let item = new_region_pin_rollup("code-17", "label-304");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
