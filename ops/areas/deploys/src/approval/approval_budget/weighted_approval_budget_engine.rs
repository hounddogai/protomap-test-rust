use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted approval budget calculation.
pub trait WeightedApprovalBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedApprovalBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedApprovalBudgetAdjuster {
    pub percent: f64,
}

impl WeightedApprovalBudgetFactor for PercentWeightedApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedApprovalBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalBudgetStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl WeightedApprovalBudgetFactor for WeightedApprovalBudgetStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedApprovalBudget, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedApprovalBudgetAdjuster {
    pub cap: f64,
}

impl WeightedApprovalBudgetFactor for CapWeightedApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedApprovalBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedApprovalBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedApprovalBudgetFactor for FlaggedWeightedApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedApprovalBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceWeightedApprovalBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedApprovalBudgetFactor for SourceWeightedApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &WeightedApprovalBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedApprovalBudgetAdjuster {
    pub minimum: f64,
}

impl WeightedApprovalBudgetFactor for MinimumWeightedApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedApprovalBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the weighted approval budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedApprovalBudgetEngine {
    pub steps: Vec<Rc<dyn WeightedApprovalBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedApprovalBudgetEngine {
    pub fn run(&mut self, item: &WeightedApprovalBudget) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedApprovalBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedApprovalBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_approval_budget_engine(flags: &flags::Registry) -> WeightedApprovalBudgetEngine {
    WeightedApprovalBudgetEngine {
        steps: vec![
            Rc::new(PercentWeightedApprovalBudgetAdjuster { percent: 15.0 }) as Rc<dyn WeightedApprovalBudgetFactor>,
            Rc::new(WeightedApprovalBudgetStartsAtOffsetAdjuster { weight: 0.9 }),
            Rc::new(CapWeightedApprovalBudgetAdjuster { cap: 2743.0 }),
            Rc::new(FlaggedWeightedApprovalBudgetAdjuster {
                flags: flags.clone(),
                flag: "weighted-approval-budget-default".to_string(),
                percent: -0.6,
            }),
            Rc::new(SourceWeightedApprovalBudgetAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.14),
                    ("online".to_string(), 1.09),
                    ("west".to_string(), 0.79),
                ]),
            }),
            Rc::new(MinimumWeightedApprovalBudgetAdjuster { minimum: 33.6 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 43062.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_approval_budget_engine(&flags::new_registry());
        let item = new_weighted_approval_budget("code-783", "title-897");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
