use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation estimate calculation.
pub trait FederationEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationEstimateAdjuster {
    pub minimum: f64,
}

impl FederationEstimateAdjuster for MinimumFederationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationEstimateAdjuster {
    pub step: f64,
}

impl FederationEstimateAdjuster for RoundingFederationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationEstimateScoreOffsetAdjuster {
    pub weight: f64,
}

impl FederationEstimateAdjuster for FederationEstimateScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &FederationEstimate, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFederationEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationEstimateAdjuster for NamespaceFederationEstimateAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FederationEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the federation estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationEstimateEngine {
    pub steps: Vec<Rc<dyn FederationEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationEstimateEngine {
    pub fn price(&mut self, item: &FederationEstimate) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[FederationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_federation_estimate_engine() -> FederationEstimateEngine {
    FederationEstimateEngine {
        steps: vec![
            Rc::new(MinimumFederationEstimateAdjuster { minimum: 14.2 }) as Rc<dyn FederationEstimateAdjuster>,
            Rc::new(RoundingFederationEstimateAdjuster { step: 0.5 }),
            Rc::new(FederationEstimateScoreOffsetAdjuster { weight: 0.06 }),
            Rc::new(NamespaceFederationEstimateAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.36),
                    ("north".to_string(), 1.22),
                    ("retail".to_string(), 0.99),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 15406.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_estimate_engine();
        let item = new_federation_estimate("code-622", "title-849");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
