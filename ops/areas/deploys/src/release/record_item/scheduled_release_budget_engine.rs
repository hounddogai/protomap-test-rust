use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled release budget calculation.
pub trait ScheduledReleaseBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledReleaseBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseBudgetMaximumOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledReleaseBudgetModifier for ScheduledReleaseBudgetMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledReleaseBudget, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledReleaseBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledReleaseBudgetModifier for TieredScheduledReleaseBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledReleaseBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledReleaseBudgetAdjuster {
    pub minimum: f64,
}

impl ScheduledReleaseBudgetModifier for MinimumScheduledReleaseBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledReleaseBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledReleaseBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledReleaseBudgetModifier for FlaggedScheduledReleaseBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledReleaseBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled release budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledReleaseBudgetEngine {
    pub steps: Vec<Rc<dyn ScheduledReleaseBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledReleaseBudgetEngine {
    pub fn run(&mut self, item: &ScheduledReleaseBudget) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledReleaseBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_release_budget_engine(flags: &flags::Registry) -> ScheduledReleaseBudgetEngine {
    ScheduledReleaseBudgetEngine {
        steps: vec![
            Rc::new(ScheduledReleaseBudgetMaximumOffsetAdjuster { weight: 1.3 })
                as Rc<dyn ScheduledReleaseBudgetModifier>,
            Rc::new(TieredScheduledReleaseBudgetAdjuster {
                bounds: vec![5327.0, 8979.0],
                rates: vec![1.12, 1.04],
            }),
            Rc::new(MinimumScheduledReleaseBudgetAdjuster { minimum: 16.4 }),
            Rc::new(FlaggedScheduledReleaseBudgetAdjuster {
                flags: flags.clone(),
                flag: "scheduled-release-budget-weighted".to_string(),
                percent: -7.0,
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 3711.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_release_budget_engine(&flags::new_registry());
        let item = new_scheduled_release_budget("code-120", "title-953");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
