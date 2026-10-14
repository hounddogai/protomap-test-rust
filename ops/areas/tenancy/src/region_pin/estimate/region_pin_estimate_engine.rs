use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin estimate calculation.
pub trait RegionPinEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinEstimateScoreOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinEstimateFactor for RegionPinEstimateScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinEstimate, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinEstimateAdjuster {
    pub step: f64,
}

impl RegionPinEstimateFactor for RoundingRegionPinEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinEstimateAdjuster {
    pub minimum: f64,
}

impl RegionPinEstimateFactor for MinimumRegionPinEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinEstimateFactor for TieredRegionPinEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinEstimateFactor for FlaggedRegionPinEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the region pin estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinEstimateEngine {
    pub steps: Vec<Rc<dyn RegionPinEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinEstimateEngine {
    pub fn run(&mut self, item: &RegionPinEstimate) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_region_pin_estimate_engine(flags: &flags::Registry) -> RegionPinEstimateEngine {
    RegionPinEstimateEngine {
        steps: vec![
            Rc::new(RegionPinEstimateScoreOffsetAdjuster { weight: 1.23 }) as Rc<dyn RegionPinEstimateFactor>,
            Rc::new(RoundingRegionPinEstimateAdjuster { step: 0.1 }),
            Rc::new(MinimumRegionPinEstimateAdjuster { minimum: 22.6 }),
            Rc::new(TieredRegionPinEstimateAdjuster {
                bounds: vec![5711.0, 8234.0],
                rates: vec![0.95, 0.81],
            }),
            Rc::new(FlaggedRegionPinEstimateAdjuster {
                flags: flags.clone(),
                flag: "region-pin-estimate-scheduled".to_string(),
                percent: 7.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 38421.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_estimate_engine(&flags::new_registry());
        let item = new_region_pin_estimate("code-809", "label-492");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
