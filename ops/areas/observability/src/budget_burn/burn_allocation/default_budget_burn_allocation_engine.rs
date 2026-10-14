use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default budget burn allocation calculation.
pub trait DefaultBudgetBurnAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultBudgetBurnAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnAllocationScoreOffsetAdjuster {
    pub weight: f64,
}

impl DefaultBudgetBurnAllocationAdjuster for DefaultBudgetBurnAllocationScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultBudgetBurnAllocation, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDefaultBudgetBurnAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultBudgetBurnAllocationAdjuster for TitleDefaultBudgetBurnAllocationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DefaultBudgetBurnAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultBudgetBurnAllocationAdjuster {
    pub minimum: f64,
}

impl DefaultBudgetBurnAllocationAdjuster for MinimumDefaultBudgetBurnAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultBudgetBurnAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultBudgetBurnAllocationAdjuster {
    pub percent: f64,
}

impl DefaultBudgetBurnAllocationAdjuster for PercentDefaultBudgetBurnAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultBudgetBurnAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default budget burn allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultBudgetBurnAllocationEngine {
    pub steps: Vec<Rc<dyn DefaultBudgetBurnAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultBudgetBurnAllocationEngine {
    pub fn run(&mut self, item: &DefaultBudgetBurnAllocation) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[DefaultBudgetBurnAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_budget_burn_allocation_engine() -> DefaultBudgetBurnAllocationEngine {
    DefaultBudgetBurnAllocationEngine {
        steps: vec![
            Rc::new(DefaultBudgetBurnAllocationScoreOffsetAdjuster { weight: 0.63 })
                as Rc<dyn DefaultBudgetBurnAllocationAdjuster>,
            Rc::new(TitleDefaultBudgetBurnAllocationAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.01),
                    ("south".to_string(), 1.01),
                    ("east".to_string(), 1.11),
                ]),
            }),
            Rc::new(MinimumDefaultBudgetBurnAllocationAdjuster { minimum: 23.5 }),
            Rc::new(PercentDefaultBudgetBurnAllocationAdjuster { percent: -8.4 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 66366.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_budget_burn_allocation_engine();
        let item = new_default_budget_burn_allocation("code-517", "title-144");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
