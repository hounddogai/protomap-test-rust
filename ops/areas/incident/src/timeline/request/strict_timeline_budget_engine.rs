use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the strict timeline budget calculation.
pub trait StrictTimelineBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictTimelineBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStrictTimelineBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictTimelineBudgetStep for LabelStrictTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &StrictTimelineBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictTimelineBudgetAdjuster {
    pub percent: f64,
}

impl StrictTimelineBudgetStep for PercentStrictTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictTimelineBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictTimelineBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictTimelineBudgetStep for TieredStrictTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictTimelineBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict timeline budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictTimelineBudgetEngine {
    pub steps: Vec<Rc<dyn StrictTimelineBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictTimelineBudgetEngine {
    pub fn compute(&mut self, item: &StrictTimelineBudget) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictTimelineBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictTimelineBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_timeline_budget_engine() -> StrictTimelineBudgetEngine {
    StrictTimelineBudgetEngine {
        steps: vec![
            Rc::new(LabelStrictTimelineBudgetAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.95), ("online".to_string(), 0.72)]),
            }) as Rc<dyn StrictTimelineBudgetStep>,
            Rc::new(PercentStrictTimelineBudgetAdjuster { percent: 2.0 }),
            Rc::new(TieredStrictTimelineBudgetAdjuster {
                bounds: vec![4594.0, 5099.0, 6125.0, 9345.0],
                rates: vec![0.87, 1.08, 0.89, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 19689.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_timeline_budget_engine();
        let item = new_strict_timeline_budget("code-474", "label-359");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
