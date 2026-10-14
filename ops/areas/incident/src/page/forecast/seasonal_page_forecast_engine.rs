use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal page forecast calculation.
pub trait SeasonalPageForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalPageForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalPageForecastAdjuster {
    pub percent: f64,
}

impl SeasonalPageForecastFactor for PercentSeasonalPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalPageForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalPageForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalPageForecastFactor for FlaggedSeasonalPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalPageForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalPageForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalPageForecastFactor for TieredSeasonalPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalPageForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal page forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalPageForecastEngine {
    pub steps: Vec<Rc<dyn SeasonalPageForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalPageForecastEngine {
    pub fn evaluate(&mut self, item: &SeasonalPageForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalPageForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_page_forecast_engine(flags: &flags::Registry) -> SeasonalPageForecastEngine {
    SeasonalPageForecastEngine {
        steps: vec![
            Rc::new(PercentSeasonalPageForecastAdjuster { percent: 9.2 }) as Rc<dyn SeasonalPageForecastFactor>,
            Rc::new(FlaggedSeasonalPageForecastAdjuster {
                flags: flags.clone(),
                flag: "seasonal-page-forecast-fallback".to_string(),
                percent: 3.3,
            }),
            Rc::new(TieredSeasonalPageForecastAdjuster {
                bounds: vec![4867.0, 7826.0, 9023.0],
                rates: vec![0.85, 1.0, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 31970.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_page_forecast_engine(&flags::new_registry());
        let item = new_seasonal_page_forecast("code-403", "label-617");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
