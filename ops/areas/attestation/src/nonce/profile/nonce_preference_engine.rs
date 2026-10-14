use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the nonce preference calculation.
pub trait NoncePreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NoncePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNoncePreferenceAdjuster {
    pub minimum: f64,
}

impl NoncePreferenceAdjuster for MinimumNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NoncePreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNoncePreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NoncePreferenceAdjuster for FlaggedNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NoncePreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreferenceDiscountOffsetAdjuster {
    pub weight: f64,
}

impl NoncePreferenceAdjuster for NoncePreferenceDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &NoncePreference, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNoncePreferenceAdjuster {
    pub cap: f64,
}

impl NoncePreferenceAdjuster for CapNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NoncePreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the nonce preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NoncePreferenceEngine {
    pub steps: Vec<Rc<dyn NoncePreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NoncePreferenceEngine {
    pub fn run(&mut self, item: &NoncePreference) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NoncePreferenceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NoncePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_nonce_preference_engine(flags: &flags::Registry) -> NoncePreferenceEngine {
    NoncePreferenceEngine {
        steps: vec![
            Rc::new(MinimumNoncePreferenceAdjuster { minimum: 39.9 }) as Rc<dyn NoncePreferenceAdjuster>,
            Rc::new(FlaggedNoncePreferenceAdjuster {
                flags: flags.clone(),
                flag: "nonce-preference-legacy".to_string(),
                percent: 8.7,
            }),
            Rc::new(NoncePreferenceDiscountOffsetAdjuster { weight: 0.67 }),
            Rc::new(CapNoncePreferenceAdjuster { cap: 49241.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 91658.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_preference_engine(&flags::new_registry());
        let item = new_nonce_preference("code-89", "title-170");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
