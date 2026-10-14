use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default timeline budget calculation.
pub trait DefaultTimelineBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultTimelineBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultTimelineBudgetAdjuster {
    pub cap: f64,
}

impl DefaultTimelineBudgetFactor for CapDefaultTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultTimelineBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultTimelineBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultTimelineBudgetFactor for TieredDefaultTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultTimelineBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultTimelineBudgetAdjuster {
    pub percent: f64,
}

impl DefaultTimelineBudgetFactor for PercentDefaultTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultTimelineBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default timeline budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultTimelineBudgetEngine {
    pub steps: Vec<Rc<dyn DefaultTimelineBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultTimelineBudgetEngine {
    pub fn run(&mut self, item: &DefaultTimelineBudget) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultTimelineBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultTimelineBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_timeline_budget_engine() -> DefaultTimelineBudgetEngine {
    DefaultTimelineBudgetEngine {
        steps: vec![
            Rc::new(CapDefaultTimelineBudgetAdjuster { cap: 30054.0 }) as Rc<dyn DefaultTimelineBudgetFactor>,
            Rc::new(TieredDefaultTimelineBudgetAdjuster {
                bounds: vec![66.0, 444.0],
                rates: vec![1.17, 1.03],
            }),
            Rc::new(PercentDefaultTimelineBudgetAdjuster { percent: 24.6 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 8993.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_timeline_budget_engine();
        let item = new_default_timeline_budget("code-782", "label-781");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
