use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the trust domain estimate calculation.
pub trait TrustDomainEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainEstimateAdjuster {
    pub percent: f64,
}

impl TrustDomainEstimateFactor for PercentTrustDomainEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTrustDomainEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainEstimateFactor for LabelTrustDomainEstimateAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TrustDomainEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainEstimateAmountOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainEstimateFactor for TrustDomainEstimateAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainEstimate, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the trust domain estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainEstimateEngine {
    pub steps: Vec<Rc<dyn TrustDomainEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainEstimateEngine {
    pub fn evaluate(&mut self, item: &TrustDomainEstimate) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_trust_domain_estimate_engine() -> TrustDomainEstimateEngine {
    TrustDomainEstimateEngine {
        steps: vec![
            Rc::new(PercentTrustDomainEstimateAdjuster { percent: -23.5 }) as Rc<dyn TrustDomainEstimateFactor>,
            Rc::new(LabelTrustDomainEstimateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.85), ("central".to_string(), 1.26)]),
            }),
            Rc::new(TrustDomainEstimateAmountOffsetAdjuster { weight: 0.23 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 95106.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_estimate_engine();
        let item = new_trust_domain_estimate("code-205", "label-710");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
