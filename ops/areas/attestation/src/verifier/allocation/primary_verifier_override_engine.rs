use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary verifier override calculation.
pub trait PrimaryVerifierOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryVerifierOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryVerifierOverrideAdjuster {
    pub percent: f64,
}

impl PrimaryVerifierOverrideModifier for PercentPrimaryVerifierOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryVerifierOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryVerifierOverrideAdjuster {
    pub step: f64,
}

impl PrimaryVerifierOverrideModifier for RoundingPrimaryVerifierOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryVerifierOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierOverrideExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryVerifierOverrideModifier for PrimaryVerifierOverrideExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryVerifierOverride, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryVerifierOverrideAdjuster {
    pub cap: f64,
}

impl PrimaryVerifierOverrideModifier for CapPrimaryVerifierOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryVerifierOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryVerifierOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryVerifierOverrideModifier for TieredPrimaryVerifierOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryVerifierOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary verifier override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierOverrideEngine {
    pub steps: Vec<Rc<dyn PrimaryVerifierOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryVerifierOverrideEngine {
    pub fn price(&mut self, item: &PrimaryVerifierOverride) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PrimaryVerifierOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_verifier_override_engine() -> PrimaryVerifierOverrideEngine {
    PrimaryVerifierOverrideEngine {
        steps: vec![
            Rc::new(PercentPrimaryVerifierOverrideAdjuster { percent: 25.0 })
                as Rc<dyn PrimaryVerifierOverrideModifier>,
            Rc::new(RoundingPrimaryVerifierOverrideAdjuster { step: 1.0 }),
            Rc::new(PrimaryVerifierOverrideExpiresAtOffsetAdjuster { weight: 1.56 }),
            Rc::new(CapPrimaryVerifierOverrideAdjuster { cap: 13315.0 }),
            Rc::new(TieredPrimaryVerifierOverrideAdjuster {
                bounds: vec![870.0, 2404.0],
                rates: vec![1.19, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 50700.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_verifier_override_engine();
        let item = new_primary_verifier_override("code-270", "title-837");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
