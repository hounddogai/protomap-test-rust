use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary query profile calculation.
pub trait PrimaryQueryProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryQueryProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryQueryProfileAdjuster {
    pub cap: f64,
}

impl PrimaryQueryProfileFactor for CapPrimaryQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryQueryProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryQueryProfileAdjuster {
    pub minimum: f64,
}

impl PrimaryQueryProfileFactor for MinimumPrimaryQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryQueryProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryQueryProfileAdjuster {
    pub step: f64,
}

impl PrimaryQueryProfileFactor for RoundingPrimaryQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryQueryProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryQueryProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryQueryProfileFactor for TieredPrimaryQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryQueryProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary query profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryQueryProfileEngine {
    pub steps: Vec<Rc<dyn PrimaryQueryProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryQueryProfileEngine {
    pub fn run(&mut self, item: &PrimaryQueryProfile) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryQueryProfileFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryQueryProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_query_profile_engine() -> PrimaryQueryProfileEngine {
    PrimaryQueryProfileEngine {
        steps: vec![
            Rc::new(CapPrimaryQueryProfileAdjuster { cap: 1535.0 }) as Rc<dyn PrimaryQueryProfileFactor>,
            Rc::new(MinimumPrimaryQueryProfileAdjuster { minimum: 47.0 }),
            Rc::new(RoundingPrimaryQueryProfileAdjuster { step: 0.1 }),
            Rc::new(TieredPrimaryQueryProfileAdjuster {
                bounds: vec![6155.0, 6903.0],
                rates: vec![1.0, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 18684.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_query_profile_engine();
        let item = new_primary_query_profile("code-337", "title-318");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
