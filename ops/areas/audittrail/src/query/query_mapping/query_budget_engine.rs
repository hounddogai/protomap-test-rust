use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the query budget calculation.
pub trait QueryBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryBudgetAdjuster {
    pub percent: f64,
}

impl QueryBudgetFactor for PercentQueryBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryBudgetFactor for FlaggedQueryBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryBudgetAdjuster {
    pub minimum: f64,
}

impl QueryBudgetFactor for MinimumQueryBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceQueryBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryBudgetFactor for SourceQueryBudgetAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &QueryBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the query budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryBudgetEngine {
    pub steps: Vec<Rc<dyn QueryBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryBudgetEngine {
    pub fn run(&mut self, item: &QueryBudget) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_query_budget_engine(flags: &flags::Registry) -> QueryBudgetEngine {
    QueryBudgetEngine {
        steps: vec![
            Rc::new(PercentQueryBudgetAdjuster { percent: -9.9 }) as Rc<dyn QueryBudgetFactor>,
            Rc::new(FlaggedQueryBudgetAdjuster {
                flags: flags.clone(),
                flag: "query-budget-tiered".to_string(),
                percent: -9.7,
            }),
            Rc::new(MinimumQueryBudgetAdjuster { minimum: 1.0 }),
            Rc::new(SourceQueryBudgetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.24), ("south".to_string(), 0.89)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 90553.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_budget_engine(&flags::new_registry());
        let item = new_query_budget("code-979", "title-738");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
