use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the fallback verifier estimate calculation.
pub trait FallbackVerifierEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackVerifierEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackVerifierEstimateAdjuster {
    pub minimum: f64,
}

impl FallbackVerifierEstimateFactor for MinimumFallbackVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackVerifierEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackVerifierEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackVerifierEstimateFactor for TieredFallbackVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackVerifierEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFallbackVerifierEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackVerifierEstimateFactor for NameFallbackVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FallbackVerifierEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierEstimateThresholdOffsetAdjuster {
    pub weight: f64,
}

impl FallbackVerifierEstimateFactor for FallbackVerifierEstimateThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackVerifierEstimate, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the fallback verifier estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackVerifierEstimateEngine {
    pub steps: Vec<Rc<dyn FallbackVerifierEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackVerifierEstimateEngine {
    pub fn price(&mut self, item: &FallbackVerifierEstimate) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackVerifierEstimateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackVerifierEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_verifier_estimate_engine() -> FallbackVerifierEstimateEngine {
    FallbackVerifierEstimateEngine {
        steps: vec![
            Rc::new(MinimumFallbackVerifierEstimateAdjuster { minimum: 46.1 })
                as Rc<dyn FallbackVerifierEstimateFactor>,
            Rc::new(TieredFallbackVerifierEstimateAdjuster {
                bounds: vec![2205.0, 4267.0, 6268.0, 7775.0],
                rates: vec![1.07, 1.13, 1.01, 0.89],
            }),
            Rc::new(NameFallbackVerifierEstimateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.8)]),
            }),
            Rc::new(FallbackVerifierEstimateThresholdOffsetAdjuster { weight: 1.14 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 20657.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_verifier_estimate_engine();
        let item = new_fallback_verifier_estimate("code-552", "name-30");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
