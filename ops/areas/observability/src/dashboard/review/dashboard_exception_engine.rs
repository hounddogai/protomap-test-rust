use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard exception calculation.
pub trait DashboardExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardExceptionAdjuster {
    pub percent: f64,
}

impl DashboardExceptionAdjuster for PercentDashboardExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardExceptionAdjuster for FlaggedDashboardExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardExceptionAdjuster {
    pub step: f64,
}

impl DashboardExceptionAdjuster for RoundingDashboardExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardExceptionAdjuster {
    pub minimum: f64,
}

impl DashboardExceptionAdjuster for MinimumDashboardExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDashboardExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardExceptionAdjuster for NameDashboardExceptionAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DashboardException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the dashboard exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardExceptionEngine {
    pub steps: Vec<Rc<dyn DashboardExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardExceptionEngine {
    pub fn price(&mut self, item: &DashboardException) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DashboardException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dashboard_exception_engine(flags: &flags::Registry) -> DashboardExceptionEngine {
    DashboardExceptionEngine {
        steps: vec![
            Rc::new(PercentDashboardExceptionAdjuster { percent: -7.4 }) as Rc<dyn DashboardExceptionAdjuster>,
            Rc::new(FlaggedDashboardExceptionAdjuster {
                flags: flags.clone(),
                flag: "dashboard-exception-batched".to_string(),
                percent: -9.6,
            }),
            Rc::new(RoundingDashboardExceptionAdjuster { step: 5.0 }),
            Rc::new(MinimumDashboardExceptionAdjuster { minimum: 23.5 }),
            Rc::new(NameDashboardExceptionAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.24), ("south".to_string(), 1.0)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 15884.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_exception_engine(&flags::new_registry());
        let item = new_dashboard_exception("code-715", "name-14");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
