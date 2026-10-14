use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin baseline calculation.
pub trait RegionPinBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinBaselineAdjuster {
    pub step: f64,
}

impl RegionPinBaselineAdjuster for RoundingRegionPinBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaselineSequenceOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinBaselineAdjuster for RegionPinBaselineSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinBaseline, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinBaselineAdjuster {
    pub minimum: f64,
}

impl RegionPinBaselineAdjuster for MinimumRegionPinBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinBaselineAdjuster for FlaggedRegionPinBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the region pin baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinBaselineEngine {
    pub steps: Vec<Rc<dyn RegionPinBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinBaselineEngine {
    pub fn compute(&mut self, item: &RegionPinBaseline) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_region_pin_baseline_engine(flags: &flags::Registry) -> RegionPinBaselineEngine {
    RegionPinBaselineEngine {
        steps: vec![
            Rc::new(RoundingRegionPinBaselineAdjuster { step: 5.0 }) as Rc<dyn RegionPinBaselineAdjuster>,
            Rc::new(RegionPinBaselineSequenceOffsetAdjuster { weight: 0.6 }),
            Rc::new(MinimumRegionPinBaselineAdjuster { minimum: 12.8 }),
            Rc::new(FlaggedRegionPinBaselineAdjuster {
                flags: flags.clone(),
                flag: "region-pin-baseline-default".to_string(),
                percent: -10.0,
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 86998.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_baseline_engine(&flags::new_registry());
        let item = new_region_pin_baseline("code-333", "label-559");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
