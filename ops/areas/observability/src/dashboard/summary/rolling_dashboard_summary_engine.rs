use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling dashboard summary calculation.
pub trait RollingDashboardSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingDashboardSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRollingDashboardSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingDashboardSummaryStep for LabelRollingDashboardSummaryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RollingDashboardSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingDashboardSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingDashboardSummaryStep for TieredRollingDashboardSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingDashboardSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingDashboardSummaryAdjuster {
    pub minimum: f64,
}

impl RollingDashboardSummaryStep for MinimumRollingDashboardSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingDashboardSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSummaryDiscountOffsetAdjuster {
    pub weight: f64,
}

impl RollingDashboardSummaryStep for RollingDashboardSummaryDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &RollingDashboardSummary, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the rolling dashboard summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingDashboardSummaryEngine {
    pub steps: Vec<Rc<dyn RollingDashboardSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingDashboardSummaryEngine {
    pub fn run(&mut self, item: &RollingDashboardSummary) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingDashboardSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingDashboardSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_dashboard_summary_engine() -> RollingDashboardSummaryEngine {
    RollingDashboardSummaryEngine {
        steps: vec![
            Rc::new(LabelRollingDashboardSummaryAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.32), ("retail".to_string(), 0.96)]),
            }) as Rc<dyn RollingDashboardSummaryStep>,
            Rc::new(TieredRollingDashboardSummaryAdjuster {
                bounds: vec![1146.0, 1586.0, 9766.0],
                rates: vec![1.0, 1.14, 1.03],
            }),
            Rc::new(MinimumRollingDashboardSummaryAdjuster { minimum: 23.6 }),
            Rc::new(RollingDashboardSummaryDiscountOffsetAdjuster { weight: 0.89 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 37346.7,
    }
}
