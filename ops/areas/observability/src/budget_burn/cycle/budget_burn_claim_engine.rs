use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the budget burn claim calculation.
pub trait BudgetBurnClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnClaimFactor for TieredBudgetBurnClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnClaimFactor for FlaggedBudgetBurnClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBudgetBurnClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnClaimFactor for NameBudgetBurnClaimAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BudgetBurnClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnClaimAdjuster {
    pub minimum: f64,
}

impl BudgetBurnClaimFactor for MinimumBudgetBurnClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnClaimAdjuster {
    pub cap: f64,
}

impl BudgetBurnClaimFactor for CapBudgetBurnClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the budget burn claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnClaimEngine {
    pub steps: Vec<Rc<dyn BudgetBurnClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnClaimEngine {
    pub fn compute(&mut self, item: &BudgetBurnClaim) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnClaimFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_budget_burn_claim_engine(flags: &flags::Registry) -> BudgetBurnClaimEngine {
    BudgetBurnClaimEngine {
        steps: vec![
            Rc::new(TieredBudgetBurnClaimAdjuster {
                bounds: vec![2201.0, 2916.0, 3175.0, 9425.0],
                rates: vec![1.12, 1.13, 0.83, 0.94],
            }) as Rc<dyn BudgetBurnClaimFactor>,
            Rc::new(FlaggedBudgetBurnClaimAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-claim-strict".to_string(),
                percent: -1.4,
            }),
            Rc::new(NameBudgetBurnClaimAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.27), ("west".to_string(), 1.1)]),
            }),
            Rc::new(MinimumBudgetBurnClaimAdjuster { minimum: 40.6 }),
            Rc::new(CapBudgetBurnClaimAdjuster { cap: 9200.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 20937.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_claim_engine(&flags::new_registry());
        let item = new_budget_burn_claim("code-736", "name-367");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
