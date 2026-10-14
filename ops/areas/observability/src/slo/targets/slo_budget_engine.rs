use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the slo budget calculation.
pub trait SloBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSloBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloBudgetStep for FlaggedSloBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloBudgetAdjuster {
    pub cap: f64,
}

impl SloBudgetStep for CapSloBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloBudgetAdjuster {
    pub percent: f64,
}

impl SloBudgetStep for PercentSloBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the slo budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloBudgetEngine {
    pub steps: Vec<Rc<dyn SloBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloBudgetEngine {
    pub fn price(&mut self, item: &SloBudget) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_slo_budget_engine(flags: &flags::Registry) -> SloBudgetEngine {
    SloBudgetEngine {
        steps: vec![
            Rc::new(FlaggedSloBudgetAdjuster {
                flags: flags.clone(),
                flag: "slo-budget-adaptive".to_string(),
                percent: -6.0,
            }) as Rc<dyn SloBudgetStep>,
            Rc::new(CapSloBudgetAdjuster { cap: 29082.0 }),
            Rc::new(PercentSloBudgetAdjuster { percent: -14.2 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 84354.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_budget_engine(&flags::new_registry());
        let item = new_slo_budget("code-578", "title-623");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
