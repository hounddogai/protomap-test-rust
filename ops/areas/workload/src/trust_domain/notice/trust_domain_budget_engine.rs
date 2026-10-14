use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the trust domain budget calculation.
pub trait TrustDomainBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainBudgetAdjuster for TieredTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTrustDomainBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainBudgetAdjuster for NameTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TrustDomainBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBudgetAmountOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainBudgetAdjuster for TrustDomainBudgetAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainBudget, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainBudgetAdjuster {
    pub minimum: f64,
}

impl TrustDomainBudgetAdjuster for MinimumTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the trust domain budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainBudgetEngine {
    pub steps: Vec<Rc<dyn TrustDomainBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainBudgetEngine {
    pub fn run(&mut self, item: &TrustDomainBudget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_trust_domain_budget_engine() -> TrustDomainBudgetEngine {
    TrustDomainBudgetEngine {
        steps: vec![
            Rc::new(TieredTrustDomainBudgetAdjuster {
                bounds: vec![2743.0, 4325.0, 9418.0],
                rates: vec![1.06, 0.96, 1.19],
            }) as Rc<dyn TrustDomainBudgetAdjuster>,
            Rc::new(NameTrustDomainBudgetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.83)]),
            }),
            Rc::new(TrustDomainBudgetAmountOffsetAdjuster { weight: 0.43 }),
            Rc::new(MinimumTrustDomainBudgetAdjuster { minimum: 17.2 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 54284.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_budget_engine();
        let item = new_trust_domain_budget("code-762", "name-52");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
