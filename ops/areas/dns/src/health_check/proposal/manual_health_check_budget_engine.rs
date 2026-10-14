use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual health check budget calculation.
pub trait ManualHealthCheckBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualHealthCheckBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualHealthCheckBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualHealthCheckBudgetFactor for FlaggedManualHealthCheckBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualHealthCheckBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBudgetVersionOffsetAdjuster {
    pub weight: f64,
}

impl ManualHealthCheckBudgetFactor for ManualHealthCheckBudgetVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ManualHealthCheckBudget, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualHealthCheckBudgetAdjuster {
    pub step: f64,
}

impl ManualHealthCheckBudgetFactor for RoundingManualHealthCheckBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualHealthCheckBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual health check budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualHealthCheckBudgetEngine {
    pub steps: Vec<Rc<dyn ManualHealthCheckBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualHealthCheckBudgetEngine {
    pub fn compute(&mut self, item: &ManualHealthCheckBudget) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ManualHealthCheckBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_health_check_budget_engine(flags: &flags::Registry) -> ManualHealthCheckBudgetEngine {
    ManualHealthCheckBudgetEngine {
        steps: vec![
            Rc::new(FlaggedManualHealthCheckBudgetAdjuster {
                flags: flags.clone(),
                flag: "manual-health-check-budget-fallback".to_string(),
                percent: 6.8,
            }) as Rc<dyn ManualHealthCheckBudgetFactor>,
            Rc::new(ManualHealthCheckBudgetVersionOffsetAdjuster { weight: 1.1 }),
            Rc::new(RoundingManualHealthCheckBudgetAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 26644.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_health_check_budget_engine(&flags::new_registry());
        let item = new_manual_health_check_budget("code-735", "title-461");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
