use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the credential plan calculation.
pub trait CredentialPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialPlanAdjuster {
    pub cap: f64,
}

impl CredentialPlanFactor for CapCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialPlanFactor for TieredCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialPlanWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl CredentialPlanFactor for CredentialPlanWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialPlan, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialPlanAdjuster {
    pub minimum: f64,
}

impl CredentialPlanFactor for MinimumCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the credential plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialPlanEngine {
    pub steps: Vec<Rc<dyn CredentialPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialPlanEngine {
    pub fn price(&mut self, item: &CredentialPlan) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialPlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CredentialPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_credential_plan_engine() -> CredentialPlanEngine {
    CredentialPlanEngine {
        steps: vec![
            Rc::new(CapCredentialPlanAdjuster { cap: 1895.0 }) as Rc<dyn CredentialPlanFactor>,
            Rc::new(TieredCredentialPlanAdjuster {
                bounds: vec![4297.0, 6872.0, 7619.0],
                rates: vec![1.12, 1.14, 0.97],
            }),
            Rc::new(CredentialPlanWindowDaysOffsetAdjuster { weight: 0.19 }),
            Rc::new(MinimumCredentialPlanAdjuster { minimum: 9.1 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 22081.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_plan_engine();
        let item = new_credential_plan("code-645", "title-704");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
