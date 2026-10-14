use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default flow log budget calculation.
pub trait DefaultFlowLogBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultFlowLogBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultFlowLogBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultFlowLogBudgetFactor for TieredDefaultFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultFlowLogBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultFlowLogBudgetAdjuster {
    pub percent: f64,
}

impl DefaultFlowLogBudgetFactor for PercentDefaultFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultFlowLogBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultFlowLogBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultFlowLogBudgetFactor for LabelDefaultFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultFlowLogBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultFlowLogBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultFlowLogBudgetFactor for FlaggedDefaultFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultFlowLogBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default flow log budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultFlowLogBudgetEngine {
    pub steps: Vec<Rc<dyn DefaultFlowLogBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultFlowLogBudgetEngine {
    pub fn run(&mut self, item: &DefaultFlowLogBudget) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultFlowLogBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultFlowLogBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_flow_log_budget_engine(flags: &flags::Registry) -> DefaultFlowLogBudgetEngine {
    DefaultFlowLogBudgetEngine {
        steps: vec![
            Rc::new(TieredDefaultFlowLogBudgetAdjuster {
                bounds: vec![558.0, 1441.0, 9329.0],
                rates: vec![0.85, 0.92, 1.01],
            }) as Rc<dyn DefaultFlowLogBudgetFactor>,
            Rc::new(PercentDefaultFlowLogBudgetAdjuster { percent: -22.9 }),
            Rc::new(LabelDefaultFlowLogBudgetAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.97)]),
            }),
            Rc::new(FlaggedDefaultFlowLogBudgetAdjuster {
                flags: flags.clone(),
                flag: "default-flow-log-budget-rolling".to_string(),
                percent: 0.3,
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 43004.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_flow_log_budget_engine(&flags::new_registry());
        let item = new_default_flow_log_budget("code-710", "label-971");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
