use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard window calculation.
pub trait DashboardWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardWindowAdjuster {
    pub step: f64,
}

impl DashboardWindowFactor for RoundingDashboardWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardWindowFactor for FlaggedDashboardWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardWindowAdjuster {
    pub minimum: f64,
}

impl DashboardWindowFactor for MinimumDashboardWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the dashboard window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardWindowEngine {
    pub steps: Vec<Rc<dyn DashboardWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardWindowEngine {
    pub fn price(&mut self, item: &DashboardWindow) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[DashboardWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dashboard_window_engine(flags: &flags::Registry) -> DashboardWindowEngine {
    DashboardWindowEngine {
        steps: vec![
            Rc::new(RoundingDashboardWindowAdjuster { step: 0.05 }) as Rc<dyn DashboardWindowFactor>,
            Rc::new(FlaggedDashboardWindowAdjuster {
                flags: flags.clone(),
                flag: "dashboard-window-seasonal".to_string(),
                percent: 9.5,
            }),
            Rc::new(MinimumDashboardWindowAdjuster { minimum: 5.3 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 27654.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_window_engine(&flags::new_registry());
        let item = new_dashboard_window("code-660", "name-660");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
