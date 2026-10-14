use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the dashboard target calculation.
pub trait DashboardTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTargetCeilingOffsetAdjuster {
    pub weight: f64,
}

impl DashboardTargetAdjuster for DashboardTargetCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardTarget, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDashboardTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardTargetAdjuster for ChannelDashboardTargetAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &DashboardTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardTargetAdjuster for TieredDashboardTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the dashboard target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardTargetEngine {
    pub steps: Vec<Rc<dyn DashboardTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardTargetEngine {
    pub fn price(&mut self, item: &DashboardTarget) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dashboard_target_engine() -> DashboardTargetEngine {
    DashboardTargetEngine {
        steps: vec![
            Rc::new(DashboardTargetCeilingOffsetAdjuster { weight: 0.74 }) as Rc<dyn DashboardTargetAdjuster>,
            Rc::new(ChannelDashboardTargetAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.73),
                    ("retail".to_string(), 1.19),
                    ("north".to_string(), 1.38),
                ]),
            }),
            Rc::new(TieredDashboardTargetAdjuster {
                bounds: vec![5173.0, 5415.0, 9659.0],
                rates: vec![1.0, 1.08, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 30814.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_target_engine();
        let item = new_dashboard_target("code-503", "title-230");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
