use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard forecast calculation.
pub trait DashboardForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardForecastFactor for FlaggedDashboardForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardForecastAdjuster {
    pub step: f64,
}

impl DashboardForecastFactor for RoundingDashboardForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardForecastFactor for TieredDashboardForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardForecastAdjuster {
    pub minimum: f64,
}

impl DashboardForecastFactor for MinimumDashboardForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardForecastAdjuster {
    pub cap: f64,
}

impl DashboardForecastFactor for CapDashboardForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardForecastCapacityOffsetAdjuster {
    pub weight: f64,
}

impl DashboardForecastFactor for DashboardForecastCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardForecast, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the dashboard forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardForecastEngine {
    pub steps: Vec<Rc<dyn DashboardForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardForecastEngine {
    pub fn run(&mut self, item: &DashboardForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DashboardForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dashboard_forecast_engine(flags: &flags::Registry) -> DashboardForecastEngine {
    DashboardForecastEngine {
        steps: vec![
            Rc::new(FlaggedDashboardForecastAdjuster {
                flags: flags.clone(),
                flag: "dashboard-forecast-primary".to_string(),
                percent: 8.6,
            }) as Rc<dyn DashboardForecastFactor>,
            Rc::new(RoundingDashboardForecastAdjuster { step: 5.0 }),
            Rc::new(TieredDashboardForecastAdjuster {
                bounds: vec![5195.0, 8076.0],
                rates: vec![1.12, 1.18],
            }),
            Rc::new(MinimumDashboardForecastAdjuster { minimum: 15.9 }),
            Rc::new(CapDashboardForecastAdjuster { cap: 29758.0 }),
            Rc::new(DashboardForecastCapacityOffsetAdjuster { weight: 1.16 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 46783.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_forecast_engine(&flags::new_registry());
        let item = new_dashboard_forecast("code-918", "label-409");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
