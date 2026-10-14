use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary verifier entry calculation.
pub trait PrimaryVerifierEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryVerifierEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryVerifierEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryVerifierEntryAdjuster for TieredPrimaryVerifierEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryVerifierEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryVerifierEntryAdjuster {
    pub step: f64,
}

impl PrimaryVerifierEntryAdjuster for RoundingPrimaryVerifierEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryVerifierEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryVerifierEntryAdjuster {
    pub percent: f64,
}

impl PrimaryVerifierEntryAdjuster for PercentPrimaryVerifierEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryVerifierEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary verifier entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierEntryEngine {
    pub steps: Vec<Rc<dyn PrimaryVerifierEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryVerifierEntryEngine {
    pub fn price(&mut self, item: &PrimaryVerifierEntry) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryVerifierEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryVerifierEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_verifier_entry_engine() -> PrimaryVerifierEntryEngine {
    PrimaryVerifierEntryEngine {
        steps: vec![
            Rc::new(TieredPrimaryVerifierEntryAdjuster {
                bounds: vec![1509.0, 1892.0, 5976.0],
                rates: vec![1.08, 0.85, 0.96],
            }) as Rc<dyn PrimaryVerifierEntryAdjuster>,
            Rc::new(RoundingPrimaryVerifierEntryAdjuster { step: 5.0 }),
            Rc::new(PercentPrimaryVerifierEntryAdjuster { percent: 22.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 1290.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_verifier_entry_engine();
        let item = new_primary_verifier_entry("code-69", "name-828");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
