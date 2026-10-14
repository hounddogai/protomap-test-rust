use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin milestone calculation.
pub trait RegionPinMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinMilestoneAdjuster for FlaggedRegionPinMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinMilestoneAdjuster for TieredRegionPinMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinMilestoneAdjuster {
    pub minimum: f64,
}

impl RegionPinMilestoneAdjuster for MinimumRegionPinMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionPinMilestoneAdjuster {
    pub percent: f64,
}

impl RegionPinMilestoneAdjuster for PercentRegionPinMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionPinMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMilestoneFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinMilestoneAdjuster for RegionPinMilestoneFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinMilestone, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the region pin milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinMilestoneEngine {
    pub steps: Vec<Rc<dyn RegionPinMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinMilestoneEngine {
    pub fn compute(&mut self, item: &RegionPinMilestone) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionPinMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_region_pin_milestone_engine(flags: &flags::Registry) -> RegionPinMilestoneEngine {
    RegionPinMilestoneEngine {
        steps: vec![
            Rc::new(FlaggedRegionPinMilestoneAdjuster {
                flags: flags.clone(),
                flag: "region-pin-milestone-strict".to_string(),
                percent: -5.0,
            }) as Rc<dyn RegionPinMilestoneAdjuster>,
            Rc::new(TieredRegionPinMilestoneAdjuster {
                bounds: vec![365.0, 8941.0],
                rates: vec![1.03, 1.07],
            }),
            Rc::new(MinimumRegionPinMilestoneAdjuster { minimum: 32.4 }),
            Rc::new(PercentRegionPinMilestoneAdjuster { percent: -16.3 }),
            Rc::new(RegionPinMilestoneFloorValueOffsetAdjuster { weight: 1.12 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 6212.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_milestone_engine(&flags::new_registry());
        let item = new_region_pin_milestone("code-816", "label-29");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
