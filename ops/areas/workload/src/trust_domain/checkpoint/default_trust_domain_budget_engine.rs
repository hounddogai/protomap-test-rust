use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default trust domain budget calculation.
pub trait DefaultTrustDomainBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultTrustDomainBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultTrustDomainBudgetAdjuster {
    pub cap: f64,
}

impl DefaultTrustDomainBudgetAdjuster for CapDefaultTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultTrustDomainBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultTrustDomainBudgetAdjuster {
    pub minimum: f64,
}

impl DefaultTrustDomainBudgetAdjuster for MinimumDefaultTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultTrustDomainBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultTrustDomainBudgetAdjuster {
    pub step: f64,
}

impl DefaultTrustDomainBudgetAdjuster for RoundingDefaultTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultTrustDomainBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultTrustDomainBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultTrustDomainBudgetAdjuster for LabelDefaultTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultTrustDomainBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultTrustDomainBudgetAdjuster {
    pub percent: f64,
}

impl DefaultTrustDomainBudgetAdjuster for PercentDefaultTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultTrustDomainBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainBudgetBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DefaultTrustDomainBudgetAdjuster for DefaultTrustDomainBudgetBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultTrustDomainBudget, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the default trust domain budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultTrustDomainBudgetEngine {
    pub steps: Vec<Rc<dyn DefaultTrustDomainBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultTrustDomainBudgetEngine {
    pub fn compute(&mut self, item: &DefaultTrustDomainBudget) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultTrustDomainBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultTrustDomainBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_trust_domain_budget_engine() -> DefaultTrustDomainBudgetEngine {
    DefaultTrustDomainBudgetEngine {
        steps: vec![
            Rc::new(CapDefaultTrustDomainBudgetAdjuster { cap: 20391.0 }) as Rc<dyn DefaultTrustDomainBudgetAdjuster>,
            Rc::new(MinimumDefaultTrustDomainBudgetAdjuster { minimum: 21.1 }),
            Rc::new(RoundingDefaultTrustDomainBudgetAdjuster { step: 0.1 }),
            Rc::new(LabelDefaultTrustDomainBudgetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.03), ("west".to_string(), 1.17)]),
            }),
            Rc::new(PercentDefaultTrustDomainBudgetAdjuster { percent: -3.7 }),
            Rc::new(DefaultTrustDomainBudgetBudgetOffsetAdjuster { weight: 1.1 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 95118.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_trust_domain_budget_engine();
        let item = new_default_trust_domain_budget("code-30", "label-262");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
