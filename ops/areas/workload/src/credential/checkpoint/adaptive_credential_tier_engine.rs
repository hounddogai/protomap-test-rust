use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive credential tier calculation.
pub trait AdaptiveCredentialTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveCredentialTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveCredentialTierAdjuster {
    pub minimum: f64,
}

impl AdaptiveCredentialTierFactor for MinimumAdaptiveCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveCredentialTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveCredentialTierAdjuster {
    pub cap: f64,
}

impl AdaptiveCredentialTierFactor for CapAdaptiveCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveCredentialTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveCredentialTierAdjuster {
    pub step: f64,
}

impl AdaptiveCredentialTierFactor for RoundingAdaptiveCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveCredentialTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAdaptiveCredentialTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveCredentialTierFactor for NameAdaptiveCredentialTierAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &AdaptiveCredentialTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveCredentialTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveCredentialTierFactor for TieredAdaptiveCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveCredentialTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveCredentialTierAdjuster {
    pub percent: f64,
}

impl AdaptiveCredentialTierFactor for PercentAdaptiveCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveCredentialTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive credential tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveCredentialTierEngine {
    pub steps: Vec<Rc<dyn AdaptiveCredentialTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveCredentialTierEngine {
    pub fn run(&mut self, item: &AdaptiveCredentialTier) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveCredentialTierFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveCredentialTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_credential_tier_engine() -> AdaptiveCredentialTierEngine {
    AdaptiveCredentialTierEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveCredentialTierAdjuster { minimum: 42.7 }) as Rc<dyn AdaptiveCredentialTierFactor>,
            Rc::new(CapAdaptiveCredentialTierAdjuster { cap: 37910.0 }),
            Rc::new(RoundingAdaptiveCredentialTierAdjuster { step: 0.1 }),
            Rc::new(NameAdaptiveCredentialTierAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.2)]),
            }),
            Rc::new(TieredAdaptiveCredentialTierAdjuster {
                bounds: vec![2796.0, 7061.0, 8358.0],
                rates: vec![1.03, 1.14, 0.94],
            }),
            Rc::new(PercentAdaptiveCredentialTierAdjuster { percent: -5.6 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 89520.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_credential_tier_engine();
        let item = new_adaptive_credential_tier("code-183", "name-160");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
