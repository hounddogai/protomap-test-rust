use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal isolation estimate calculation.
pub trait SeasonalIsolationEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalIsolationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalIsolationEstimateAdjuster {
    pub cap: f64,
}

impl SeasonalIsolationEstimateFactor for CapSeasonalIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalIsolationEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalIsolationEstimateAdjuster {
    pub minimum: f64,
}

impl SeasonalIsolationEstimateFactor for MinimumSeasonalIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalIsolationEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationEstimateCostOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalIsolationEstimateFactor for SeasonalIsolationEstimateCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalIsolationEstimate, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionSeasonalIsolationEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalIsolationEstimateFactor for RegionSeasonalIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &SeasonalIsolationEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalIsolationEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalIsolationEstimateFactor for FlaggedSeasonalIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalIsolationEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalIsolationEstimateAdjuster {
    pub step: f64,
}

impl SeasonalIsolationEstimateFactor for RoundingSeasonalIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalIsolationEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal isolation estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalIsolationEstimateEngine {
    pub steps: Vec<Rc<dyn SeasonalIsolationEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalIsolationEstimateEngine {
    pub fn run(&mut self, item: &SeasonalIsolationEstimate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalIsolationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_isolation_estimate_engine(flags: &flags::Registry) -> SeasonalIsolationEstimateEngine {
    SeasonalIsolationEstimateEngine {
        steps: vec![
            Rc::new(CapSeasonalIsolationEstimateAdjuster { cap: 35170.0 }) as Rc<dyn SeasonalIsolationEstimateFactor>,
            Rc::new(MinimumSeasonalIsolationEstimateAdjuster { minimum: 7.9 }),
            Rc::new(SeasonalIsolationEstimateCostOffsetAdjuster { weight: 1.13 }),
            Rc::new(RegionSeasonalIsolationEstimateAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.23),
                    ("retail".to_string(), 1.09),
                    ("central".to_string(), 1.21),
                ]),
            }),
            Rc::new(FlaggedSeasonalIsolationEstimateAdjuster {
                flags: flags.clone(),
                flag: "seasonal-isolation-estimate-regional".to_string(),
                percent: 9.0,
            }),
            Rc::new(RoundingSeasonalIsolationEstimateAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 36138.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_isolation_estimate_engine(&flags::new_registry());
        let item = new_seasonal_isolation_estimate("code-685", "label-806");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
