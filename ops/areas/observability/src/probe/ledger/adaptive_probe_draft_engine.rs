use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive probe draft calculation.
pub trait AdaptiveProbeDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveProbeDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveProbeDraftAdjuster {
    pub minimum: f64,
}

impl AdaptiveProbeDraftFactor for MinimumAdaptiveProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveProbeDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraftQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveProbeDraftFactor for AdaptiveProbeDraftQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveProbeDraft, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveProbeDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveProbeDraftFactor for TieredAdaptiveProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveProbeDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive probe draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveProbeDraftEngine {
    pub steps: Vec<Rc<dyn AdaptiveProbeDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveProbeDraftEngine {
    pub fn evaluate(&mut self, item: &AdaptiveProbeDraft) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveProbeDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_probe_draft_engine() -> AdaptiveProbeDraftEngine {
    AdaptiveProbeDraftEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveProbeDraftAdjuster { minimum: 4.1 }) as Rc<dyn AdaptiveProbeDraftFactor>,
            Rc::new(AdaptiveProbeDraftQuotaUnitsOffsetAdjuster { weight: 1.71 }),
            Rc::new(TieredAdaptiveProbeDraftAdjuster {
                bounds: vec![1761.0, 3438.0],
                rates: vec![1.02, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 71515.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_probe_draft_engine();
        let item = new_adaptive_probe_draft("code-132", "label-698");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
