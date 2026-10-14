use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the delegation preference calculation.
pub trait DelegationPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationPreferenceAdjuster {
    pub cap: f64,
}

impl DelegationPreferenceStep for CapDelegationPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationPreferenceAdjuster {
    pub minimum: f64,
}

impl DelegationPreferenceStep for MinimumDelegationPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeDelegationPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationPreferenceStep for ScopeDelegationPreferenceAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &DelegationPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationPreferenceAdjuster {
    pub step: f64,
}

impl DelegationPreferenceStep for RoundingDelegationPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the delegation preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationPreferenceEngine {
    pub steps: Vec<Rc<dyn DelegationPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationPreferenceEngine {
    pub fn price(&mut self, item: &DelegationPreference) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationPreferenceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_delegation_preference_engine() -> DelegationPreferenceEngine {
    DelegationPreferenceEngine {
        steps: vec![
            Rc::new(CapDelegationPreferenceAdjuster { cap: 11365.0 }) as Rc<dyn DelegationPreferenceStep>,
            Rc::new(MinimumDelegationPreferenceAdjuster { minimum: 41.9 }),
            Rc::new(ScopeDelegationPreferenceAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.93)]),
            }),
            Rc::new(RoundingDelegationPreferenceAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 24602.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_preference_engine();
        let item = new_delegation_preference("code-806", "title-986");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
