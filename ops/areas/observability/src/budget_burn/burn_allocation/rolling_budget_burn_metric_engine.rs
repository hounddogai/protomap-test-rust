use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling budget burn metric calculation.
pub trait RollingBudgetBurnMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingBudgetBurnMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingBudgetBurnMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingBudgetBurnMetricFactor for TieredRollingBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingBudgetBurnMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingBudgetBurnMetricAdjuster {
    pub percent: f64,
}

impl RollingBudgetBurnMetricFactor for PercentRollingBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingBudgetBurnMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingBudgetBurnMetricAdjuster {
    pub step: f64,
}

impl RollingBudgetBurnMetricFactor for RoundingRollingBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingBudgetBurnMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetricWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl RollingBudgetBurnMetricFactor for RollingBudgetBurnMetricWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &RollingBudgetBurnMetric, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingBudgetBurnMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingBudgetBurnMetricFactor for TitleRollingBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingBudgetBurnMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling budget burn metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingBudgetBurnMetricEngine {
    pub steps: Vec<Rc<dyn RollingBudgetBurnMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingBudgetBurnMetricEngine {
    pub fn price(&mut self, item: &RollingBudgetBurnMetric) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingBudgetBurnMetricFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RollingBudgetBurnMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_budget_burn_metric_engine() -> RollingBudgetBurnMetricEngine {
    RollingBudgetBurnMetricEngine {
        steps: vec![
            Rc::new(TieredRollingBudgetBurnMetricAdjuster {
                bounds: vec![3917.0, 7440.0, 7473.0],
                rates: vec![1.2, 1.2, 0.91],
            }) as Rc<dyn RollingBudgetBurnMetricFactor>,
            Rc::new(PercentRollingBudgetBurnMetricAdjuster { percent: 4.4 }),
            Rc::new(RoundingRollingBudgetBurnMetricAdjuster { step: 1.0 }),
            Rc::new(RollingBudgetBurnMetricWindowDaysOffsetAdjuster { weight: 0.18 }),
            Rc::new(TitleRollingBudgetBurnMetricAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.17)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 67181.1,
    }
}
