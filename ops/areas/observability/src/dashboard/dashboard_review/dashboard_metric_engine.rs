use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dashboard metric calculation.
pub trait DashboardMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardMetricAdjuster {
    pub cap: f64,
}

impl DashboardMetricModifier for CapDashboardMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMetricWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl DashboardMetricModifier for DashboardMetricWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardMetric, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardMetricModifier for TieredDashboardMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardMetricAdjuster {
    pub minimum: f64,
}

impl DashboardMetricModifier for MinimumDashboardMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardMetricAdjuster {
    pub percent: f64,
}

impl DashboardMetricModifier for PercentDashboardMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dashboard metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardMetricEngine {
    pub steps: Vec<Rc<dyn DashboardMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardMetricEngine {
    pub fn price(&mut self, item: &DashboardMetric) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[DashboardMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dashboard_metric_engine() -> DashboardMetricEngine {
    DashboardMetricEngine {
        steps: vec![
            Rc::new(CapDashboardMetricAdjuster { cap: 22158.0 }) as Rc<dyn DashboardMetricModifier>,
            Rc::new(DashboardMetricWindowDaysOffsetAdjuster { weight: 0.18 }),
            Rc::new(TieredDashboardMetricAdjuster {
                bounds: vec![116.0, 514.0, 4225.0],
                rates: vec![1.01, 1.18, 1.2],
            }),
            Rc::new(MinimumDashboardMetricAdjuster { minimum: 42.1 }),
            Rc::new(PercentDashboardMetricAdjuster { percent: 5.4 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 67235.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_metric_engine();
        let item = new_dashboard_metric("code-370", "title-938");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
