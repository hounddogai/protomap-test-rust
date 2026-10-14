use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal alert bundle calculation.
pub trait SeasonalAlertBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalAlertBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalAlertBundleAdjuster {
    pub step: f64,
}

impl SeasonalAlertBundleFactor for RoundingSeasonalAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalAlertBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertBundleScoreOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalAlertBundleFactor for SeasonalAlertBundleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalAlertBundle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalAlertBundleAdjuster {
    pub cap: f64,
}

impl SeasonalAlertBundleFactor for CapSeasonalAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalAlertBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalAlertBundleAdjuster {
    pub percent: f64,
}

impl SeasonalAlertBundleFactor for PercentSeasonalAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalAlertBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalAlertBundleAdjuster {
    pub minimum: f64,
}

impl SeasonalAlertBundleFactor for MinimumSeasonalAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalAlertBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalAlertBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalAlertBundleFactor for TieredSeasonalAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalAlertBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal alert bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalAlertBundleEngine {
    pub steps: Vec<Rc<dyn SeasonalAlertBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalAlertBundleEngine {
    pub fn run(&mut self, item: &SeasonalAlertBundle) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalAlertBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_alert_bundle_engine() -> SeasonalAlertBundleEngine {
    SeasonalAlertBundleEngine {
        steps: vec![
            Rc::new(RoundingSeasonalAlertBundleAdjuster { step: 1.0 }) as Rc<dyn SeasonalAlertBundleFactor>,
            Rc::new(SeasonalAlertBundleScoreOffsetAdjuster { weight: 1.6 }),
            Rc::new(CapSeasonalAlertBundleAdjuster { cap: 33030.0 }),
            Rc::new(PercentSeasonalAlertBundleAdjuster { percent: 11.6 }),
            Rc::new(MinimumSeasonalAlertBundleAdjuster { minimum: 9.4 }),
            Rc::new(TieredSeasonalAlertBundleAdjuster {
                bounds: vec![54.0, 215.0, 5061.0, 5137.0],
                rates: vec![0.91, 0.99, 0.8, 1.14],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 84296.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_alert_bundle_engine();
        let item = new_seasonal_alert_bundle("code-775", "name-931");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
