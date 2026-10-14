use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default freeze profile calculation.
pub trait DefaultFreezeProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultFreezeProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultFreezeProfileAdjuster {
    pub step: f64,
}

impl DefaultFreezeProfileModifier for RoundingDefaultFreezeProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultFreezeProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultFreezeProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultFreezeProfileModifier for TieredDefaultFreezeProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultFreezeProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultFreezeProfileAdjuster {
    pub minimum: f64,
}

impl DefaultFreezeProfileModifier for MinimumDefaultFreezeProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultFreezeProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultFreezeProfileAdjuster {
    pub cap: f64,
}

impl DefaultFreezeProfileModifier for CapDefaultFreezeProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultFreezeProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default freeze profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultFreezeProfileEngine {
    pub steps: Vec<Rc<dyn DefaultFreezeProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultFreezeProfileEngine {
    pub fn run(&mut self, item: &DefaultFreezeProfile) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultFreezeProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultFreezeProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_freeze_profile_engine() -> DefaultFreezeProfileEngine {
    DefaultFreezeProfileEngine {
        steps: vec![
            Rc::new(RoundingDefaultFreezeProfileAdjuster { step: 0.1 }) as Rc<dyn DefaultFreezeProfileModifier>,
            Rc::new(TieredDefaultFreezeProfileAdjuster {
                bounds: vec![4483.0, 9306.0],
                rates: vec![1.2, 1.11],
            }),
            Rc::new(MinimumDefaultFreezeProfileAdjuster { minimum: 18.1 }),
            Rc::new(CapDefaultFreezeProfileAdjuster { cap: 7974.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 64156.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_freeze_profile_engine();
        let item = new_default_freeze_profile("code-651", "name-124");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
