use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped peering budget calculation.
pub trait CappedPeeringBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPeeringBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedPeeringBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedPeeringBudgetAdjuster for FlaggedCappedPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedPeeringBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleCappedPeeringBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedPeeringBudgetAdjuster for LocaleCappedPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &CappedPeeringBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedPeeringBudgetAdjuster {
    pub minimum: f64,
}

impl CappedPeeringBudgetAdjuster for MinimumCappedPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedPeeringBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedPeeringBudgetAdjuster {
    pub cap: f64,
}

impl CappedPeeringBudgetAdjuster for CapCappedPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedPeeringBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedPeeringBudgetAdjuster {
    pub step: f64,
}

impl CappedPeeringBudgetAdjuster for RoundingCappedPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedPeeringBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the capped peering budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPeeringBudgetEngine {
    pub steps: Vec<Rc<dyn CappedPeeringBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPeeringBudgetEngine {
    pub fn evaluate(&mut self, item: &CappedPeeringBudget) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedPeeringBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedPeeringBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_peering_budget_engine(flags: &flags::Registry) -> CappedPeeringBudgetEngine {
    CappedPeeringBudgetEngine {
        steps: vec![
            Rc::new(FlaggedCappedPeeringBudgetAdjuster {
                flags: flags.clone(),
                flag: "capped-peering-budget-primary".to_string(),
                percent: -0.9,
            }) as Rc<dyn CappedPeeringBudgetAdjuster>,
            Rc::new(LocaleCappedPeeringBudgetAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.22)]),
            }),
            Rc::new(MinimumCappedPeeringBudgetAdjuster { minimum: 13.0 }),
            Rc::new(CapCappedPeeringBudgetAdjuster { cap: 7902.0 }),
            Rc::new(RoundingCappedPeeringBudgetAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 90677.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_peering_budget_engine(&flags::new_registry());
        let item = new_capped_peering_budget("code-839", "label-579");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
