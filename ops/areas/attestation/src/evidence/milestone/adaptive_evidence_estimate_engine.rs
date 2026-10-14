use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive evidence estimate calculation.
pub trait AdaptiveEvidenceEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveEvidenceEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveEvidenceEstimateAdjuster {
    pub percent: f64,
}

impl AdaptiveEvidenceEstimateStep for PercentAdaptiveEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveEvidenceEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveEvidenceEstimateAdjuster {
    pub step: f64,
}

impl AdaptiveEvidenceEstimateStep for RoundingAdaptiveEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveEvidenceEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceEstimateCostOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveEvidenceEstimateStep for AdaptiveEvidenceEstimateCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveEvidenceEstimate, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveEvidenceEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveEvidenceEstimateStep for TieredAdaptiveEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveEvidenceEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive evidence estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveEvidenceEstimateEngine {
    pub steps: Vec<Rc<dyn AdaptiveEvidenceEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveEvidenceEstimateEngine {
    pub fn run(&mut self, item: &AdaptiveEvidenceEstimate) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveEvidenceEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_evidence_estimate_engine() -> AdaptiveEvidenceEstimateEngine {
    AdaptiveEvidenceEstimateEngine {
        steps: vec![
            Rc::new(PercentAdaptiveEvidenceEstimateAdjuster { percent: 14.9 }) as Rc<dyn AdaptiveEvidenceEstimateStep>,
            Rc::new(RoundingAdaptiveEvidenceEstimateAdjuster { step: 0.1 }),
            Rc::new(AdaptiveEvidenceEstimateCostOffsetAdjuster { weight: 0.78 }),
            Rc::new(TieredAdaptiveEvidenceEstimateAdjuster {
                bounds: vec![1392.0, 5698.0],
                rates: vec![0.9, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 4810.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_evidence_estimate_engine();
        let item = new_adaptive_evidence_estimate("code-273", "label-838");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
