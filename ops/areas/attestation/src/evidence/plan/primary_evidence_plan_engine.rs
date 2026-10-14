use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary evidence plan calculation.
pub trait PrimaryEvidencePlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEvidencePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryEvidencePlanAdjuster {
    pub percent: f64,
}

impl PrimaryEvidencePlanFactor for PercentPrimaryEvidencePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryEvidencePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryEvidencePlanAdjuster {
    pub step: f64,
}

impl PrimaryEvidencePlanFactor for RoundingPrimaryEvidencePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryEvidencePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryEvidencePlanAdjuster {
    pub minimum: f64,
}

impl PrimaryEvidencePlanFactor for MinimumPrimaryEvidencePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryEvidencePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryEvidencePlanAdjuster {
    pub cap: f64,
}

impl PrimaryEvidencePlanFactor for CapPrimaryEvidencePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryEvidencePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryEvidencePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryEvidencePlanFactor for TieredPrimaryEvidencePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryEvidencePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary evidence plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEvidencePlanEngine {
    pub steps: Vec<Rc<dyn PrimaryEvidencePlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEvidencePlanEngine {
    pub fn price(&mut self, item: &PrimaryEvidencePlan) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEvidencePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_evidence_plan_engine() -> PrimaryEvidencePlanEngine {
    PrimaryEvidencePlanEngine {
        steps: vec![
            Rc::new(PercentPrimaryEvidencePlanAdjuster { percent: 22.6 }) as Rc<dyn PrimaryEvidencePlanFactor>,
            Rc::new(RoundingPrimaryEvidencePlanAdjuster { step: 5.0 }),
            Rc::new(MinimumPrimaryEvidencePlanAdjuster { minimum: 9.8 }),
            Rc::new(CapPrimaryEvidencePlanAdjuster { cap: 15050.0 }),
            Rc::new(TieredPrimaryEvidencePlanAdjuster {
                bounds: vec![1686.0, 4112.0, 7784.0],
                rates: vec![0.85, 0.82, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 69085.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_evidence_plan_engine();
        let item = new_primary_evidence_plan("code-366", "name-183");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
