use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal release estimate calculation.
pub trait SeasonalReleaseEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalReleaseEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalReleaseEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalReleaseEstimateFactor for TieredSeasonalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalReleaseEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleSeasonalReleaseEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalReleaseEstimateFactor for LocaleSeasonalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &SeasonalReleaseEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalReleaseEstimateAdjuster {
    pub step: f64,
}

impl SeasonalReleaseEstimateFactor for RoundingSeasonalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalReleaseEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalReleaseEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalReleaseEstimateFactor for FlaggedSeasonalReleaseEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalReleaseEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal release estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalReleaseEstimateEngine {
    pub steps: Vec<Rc<dyn SeasonalReleaseEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalReleaseEstimateEngine {
    pub fn run(&mut self, item: &SeasonalReleaseEstimate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalReleaseEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_release_estimate_engine(flags: &flags::Registry) -> SeasonalReleaseEstimateEngine {
    SeasonalReleaseEstimateEngine {
        steps: vec![
            Rc::new(TieredSeasonalReleaseEstimateAdjuster {
                bounds: vec![2959.0, 4403.0],
                rates: vec![1.0, 0.85],
            }) as Rc<dyn SeasonalReleaseEstimateFactor>,
            Rc::new(LocaleSeasonalReleaseEstimateAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.88), ("online".to_string(), 0.82)]),
            }),
            Rc::new(RoundingSeasonalReleaseEstimateAdjuster { step: 0.25 }),
            Rc::new(FlaggedSeasonalReleaseEstimateAdjuster {
                flags: flags.clone(),
                flag: "seasonal-release-estimate-manual".to_string(),
                percent: -5.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 7711.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_release_estimate_engine(&flags::new_registry());
        let item = new_seasonal_release_estimate("code-331", "name-145");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
