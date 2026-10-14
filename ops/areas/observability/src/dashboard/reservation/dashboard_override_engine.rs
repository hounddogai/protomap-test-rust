use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the dashboard override calculation.
pub trait DashboardOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardOverrideAdjuster {
    pub percent: f64,
}

impl DashboardOverrideAdjuster for PercentDashboardOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardOverrideAdjuster {
    pub cap: f64,
}

impl DashboardOverrideAdjuster for CapDashboardOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardOverrideAdjuster for TieredDashboardOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the dashboard override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardOverrideEngine {
    pub steps: Vec<Rc<dyn DashboardOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardOverrideEngine {
    pub fn evaluate(&mut self, item: &DashboardOverride) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dashboard_override_engine() -> DashboardOverrideEngine {
    DashboardOverrideEngine {
        steps: vec![
            Rc::new(PercentDashboardOverrideAdjuster { percent: 19.2 }) as Rc<dyn DashboardOverrideAdjuster>,
            Rc::new(CapDashboardOverrideAdjuster { cap: 29799.0 }),
            Rc::new(TieredDashboardOverrideAdjuster {
                bounds: vec![1531.0, 2969.0],
                rates: vec![1.17, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 77199.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_override_engine();
        let item = new_dashboard_override("code-664", "label-494");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
