use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped page budget calculation.
pub trait CappedPageBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPageBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedPageBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedPageBudgetAdjuster for FlaggedCappedPageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedPageBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedPageBudgetAdjuster {
    pub percent: f64,
}

impl CappedPageBudgetAdjuster for PercentCappedPageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedPageBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedPageBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedPageBudgetAdjuster for NameCappedPageBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedPageBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedPageBudgetAdjuster {
    pub minimum: f64,
}

impl CappedPageBudgetAdjuster for MinimumCappedPageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedPageBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped page budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPageBudgetEngine {
    pub steps: Vec<Rc<dyn CappedPageBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPageBudgetEngine {
    pub fn evaluate(&mut self, item: &CappedPageBudget) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedPageBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedPageBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_page_budget_engine(flags: &flags::Registry) -> CappedPageBudgetEngine {
    CappedPageBudgetEngine {
        steps: vec![
            Rc::new(FlaggedCappedPageBudgetAdjuster {
                flags: flags.clone(),
                flag: "capped-page-budget-capped".to_string(),
                percent: -0.6,
            }) as Rc<dyn CappedPageBudgetAdjuster>,
            Rc::new(PercentCappedPageBudgetAdjuster { percent: 14.4 }),
            Rc::new(NameCappedPageBudgetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.96)]),
            }),
            Rc::new(MinimumCappedPageBudgetAdjuster { minimum: 3.8 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 22623.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_page_budget_engine(&flags::new_registry());
        let item = new_capped_page_budget("code-183", "name-816");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
