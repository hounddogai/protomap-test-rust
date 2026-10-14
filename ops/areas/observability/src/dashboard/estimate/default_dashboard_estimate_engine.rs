use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default dashboard estimate calculation.
pub trait DefaultDashboardEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultDashboardEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultDashboardEstimateAdjuster {
    pub cap: f64,
}

impl DefaultDashboardEstimateStep for CapDefaultDashboardEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultDashboardEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardEstimateRevisionOffsetAdjuster {
    pub weight: f64,
}

impl DefaultDashboardEstimateStep for DefaultDashboardEstimateRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultDashboardEstimate, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultDashboardEstimateAdjuster {
    pub percent: f64,
}

impl DefaultDashboardEstimateStep for PercentDefaultDashboardEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultDashboardEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDefaultDashboardEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultDashboardEstimateStep for TitleDefaultDashboardEstimateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DefaultDashboardEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultDashboardEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultDashboardEstimateStep for TieredDefaultDashboardEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultDashboardEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default dashboard estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultDashboardEstimateEngine {
    pub steps: Vec<Rc<dyn DefaultDashboardEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultDashboardEstimateEngine {
    pub fn run(&mut self, item: &DefaultDashboardEstimate) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultDashboardEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_dashboard_estimate_engine() -> DefaultDashboardEstimateEngine {
    DefaultDashboardEstimateEngine {
        steps: vec![
            Rc::new(CapDefaultDashboardEstimateAdjuster { cap: 39657.0 }) as Rc<dyn DefaultDashboardEstimateStep>,
            Rc::new(DefaultDashboardEstimateRevisionOffsetAdjuster { weight: 1.48 }),
            Rc::new(PercentDefaultDashboardEstimateAdjuster { percent: 6.2 }),
            Rc::new(TitleDefaultDashboardEstimateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.01), ("online".to_string(), 0.91)]),
            }),
            Rc::new(TieredDefaultDashboardEstimateAdjuster {
                bounds: vec![5305.0, 5591.0, 7149.0, 7899.0],
                rates: vec![1.09, 1.0, 1.17, 1.19],
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 80337.2,
    }
}
