use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional runbook forecast calculation.
pub trait RegionalRunbookForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRunbookForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookForecastScoreOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRunbookForecastModifier for RegionalRunbookForecastScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRunbookForecast, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalRunbookForecastAdjuster {
    pub step: f64,
}

impl RegionalRunbookForecastModifier for RoundingRegionalRunbookForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalRunbookForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalRunbookForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalRunbookForecastModifier for FlaggedRegionalRunbookForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalRunbookForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalRunbookForecastAdjuster {
    pub percent: f64,
}

impl RegionalRunbookForecastModifier for PercentRegionalRunbookForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalRunbookForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalRunbookForecastAdjuster {
    pub cap: f64,
}

impl RegionalRunbookForecastModifier for CapRegionalRunbookForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalRunbookForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional runbook forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRunbookForecastEngine {
    pub steps: Vec<Rc<dyn RegionalRunbookForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRunbookForecastEngine {
    pub fn price(&mut self, item: &RegionalRunbookForecast) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRunbookForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_runbook_forecast_engine(flags: &flags::Registry) -> RegionalRunbookForecastEngine {
    RegionalRunbookForecastEngine {
        steps: vec![
            Rc::new(RegionalRunbookForecastScoreOffsetAdjuster { weight: 1.92 })
                as Rc<dyn RegionalRunbookForecastModifier>,
            Rc::new(RoundingRegionalRunbookForecastAdjuster { step: 0.25 }),
            Rc::new(FlaggedRegionalRunbookForecastAdjuster {
                flags: flags.clone(),
                flag: "regional-runbook-forecast-regional".to_string(),
                percent: 2.0,
            }),
            Rc::new(PercentRegionalRunbookForecastAdjuster { percent: -2.8 }),
            Rc::new(CapRegionalRunbookForecastAdjuster { cap: 34254.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 64494.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_runbook_forecast_engine(&flags::new_registry());
        let item = new_regional_runbook_forecast("code-471", "label-913");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
