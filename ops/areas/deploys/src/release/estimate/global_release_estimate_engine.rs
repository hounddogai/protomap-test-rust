use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global release estimate calculation.
pub trait GlobalReleaseEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReleaseEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalReleaseEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalReleaseEstimateFactor for FlaggedGlobalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalReleaseEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalReleaseEstimateAdjuster {
    pub percent: f64,
}

impl GlobalReleaseEstimateFactor for PercentGlobalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalReleaseEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalReleaseEstimateAdjuster {
    pub minimum: f64,
}

impl GlobalReleaseEstimateFactor for MinimumGlobalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalReleaseEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEstimateEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalReleaseEstimateFactor for GlobalReleaseEstimateEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalReleaseEstimate, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalReleaseEstimateAdjuster {
    pub step: f64,
}

impl GlobalReleaseEstimateFactor for RoundingGlobalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalReleaseEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalReleaseEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalReleaseEstimateFactor for TieredGlobalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalReleaseEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the global release estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReleaseEstimateEngine {
    pub steps: Vec<Rc<dyn GlobalReleaseEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReleaseEstimateEngine {
    pub fn run(&mut self, item: &GlobalReleaseEstimate) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReleaseEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_release_estimate_engine(flags: &flags::Registry) -> GlobalReleaseEstimateEngine {
    GlobalReleaseEstimateEngine {
        steps: vec![
            Rc::new(FlaggedGlobalReleaseEstimateAdjuster {
                flags: flags.clone(),
                flag: "global-release-estimate-fallback".to_string(),
                percent: -2.3,
            }) as Rc<dyn GlobalReleaseEstimateFactor>,
            Rc::new(PercentGlobalReleaseEstimateAdjuster { percent: 19.5 }),
            Rc::new(MinimumGlobalReleaseEstimateAdjuster { minimum: 28.9 }),
            Rc::new(GlobalReleaseEstimateEndsAtOffsetAdjuster { weight: 0.9 }),
            Rc::new(RoundingGlobalReleaseEstimateAdjuster { step: 5.0 }),
            Rc::new(TieredGlobalReleaseEstimateAdjuster {
                bounds: vec![4713.0, 5161.0, 9192.0],
                rates: vec![0.9, 0.82, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 67379.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_release_estimate_engine(&flags::new_registry());
        let item = new_global_release_estimate("code-538", "name-769");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
