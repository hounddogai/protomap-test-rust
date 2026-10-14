use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity preference calculation.
pub trait IdentityPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityPreferenceAdjuster {
    pub step: f64,
}

impl IdentityPreferenceAdjuster for RoundingIdentityPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityPreferenceAdjuster for TieredIdentityPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPreferenceBudgetOffsetAdjuster {
    pub weight: f64,
}

impl IdentityPreferenceAdjuster for IdentityPreferenceBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityPreference, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityPreferenceAdjuster {
    pub cap: f64,
}

impl IdentityPreferenceAdjuster for CapIdentityPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityPreferenceAdjuster {
    pub percent: f64,
}

impl IdentityPreferenceAdjuster for PercentIdentityPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityPreferenceEngine {
    pub steps: Vec<Rc<dyn IdentityPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityPreferenceEngine {
    pub fn compute(&mut self, item: &IdentityPreference) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityPreferenceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_preference_engine() -> IdentityPreferenceEngine {
    IdentityPreferenceEngine {
        steps: vec![
            Rc::new(RoundingIdentityPreferenceAdjuster { step: 1.0 }) as Rc<dyn IdentityPreferenceAdjuster>,
            Rc::new(TieredIdentityPreferenceAdjuster {
                bounds: vec![2578.0, 5011.0, 7039.0, 9327.0],
                rates: vec![0.92, 0.97, 0.82, 1.01],
            }),
            Rc::new(IdentityPreferenceBudgetOffsetAdjuster { weight: 1.21 }),
            Rc::new(CapIdentityPreferenceAdjuster { cap: 9103.0 }),
            Rc::new(PercentIdentityPreferenceAdjuster { percent: 24.4 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 16385.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_preference_engine();
        let item = new_identity_preference("code-786", "name-645");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
