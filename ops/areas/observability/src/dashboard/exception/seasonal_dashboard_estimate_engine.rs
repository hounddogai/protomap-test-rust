use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal dashboard estimate calculation.
pub trait SeasonalDashboardEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalDashboardEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalDashboardEstimateAdjuster {
    pub minimum: f64,
}

impl SeasonalDashboardEstimateStep for MinimumSeasonalDashboardEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalDashboardEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalDashboardEstimateAdjuster {
    pub step: f64,
}

impl SeasonalDashboardEstimateStep for RoundingSeasonalDashboardEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalDashboardEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionSeasonalDashboardEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalDashboardEstimateStep for RegionSeasonalDashboardEstimateAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &SeasonalDashboardEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardEstimatePortOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalDashboardEstimateStep for SeasonalDashboardEstimatePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalDashboardEstimate, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalDashboardEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalDashboardEstimateStep for TieredSeasonalDashboardEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalDashboardEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal dashboard estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalDashboardEstimateEngine {
    pub steps: Vec<Rc<dyn SeasonalDashboardEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalDashboardEstimateEngine {
    pub fn price(&mut self, item: &SeasonalDashboardEstimate) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalDashboardEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_dashboard_estimate_engine() -> SeasonalDashboardEstimateEngine {
    SeasonalDashboardEstimateEngine {
        steps: vec![
            Rc::new(MinimumSeasonalDashboardEstimateAdjuster { minimum: 28.8 })
                as Rc<dyn SeasonalDashboardEstimateStep>,
            Rc::new(RoundingSeasonalDashboardEstimateAdjuster { step: 0.05 }),
            Rc::new(RegionSeasonalDashboardEstimateAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.28)]),
            }),
            Rc::new(SeasonalDashboardEstimatePortOffsetAdjuster { weight: 1.67 }),
            Rc::new(TieredSeasonalDashboardEstimateAdjuster {
                bounds: vec![594.0, 1310.0, 1594.0],
                rates: vec![1.16, 1.04, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 62770.6,
    }
}
