use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive approval agreement calculation.
pub trait AdaptiveApprovalAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveApprovalAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveApprovalAgreementAdjuster {
    pub step: f64,
}

impl AdaptiveApprovalAgreementFactor for RoundingAdaptiveApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveApprovalAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveApprovalAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveApprovalAgreementFactor for TieredAdaptiveApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveApprovalAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveApprovalAgreementAdjuster {
    pub minimum: f64,
}

impl AdaptiveApprovalAgreementFactor for MinimumAdaptiveApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveApprovalAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveApprovalAgreementAdjuster {
    pub percent: f64,
}

impl AdaptiveApprovalAgreementFactor for PercentAdaptiveApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveApprovalAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveApprovalAgreementCeilingOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveApprovalAgreementFactor for AdaptiveApprovalAgreementCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveApprovalAgreement, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the adaptive approval agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveApprovalAgreementEngine {
    pub steps: Vec<Rc<dyn AdaptiveApprovalAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveApprovalAgreementEngine {
    pub fn price(&mut self, item: &AdaptiveApprovalAgreement) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveApprovalAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveApprovalAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_approval_agreement_engine() -> AdaptiveApprovalAgreementEngine {
    AdaptiveApprovalAgreementEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveApprovalAgreementAdjuster { step: 0.1 }) as Rc<dyn AdaptiveApprovalAgreementFactor>,
            Rc::new(TieredAdaptiveApprovalAgreementAdjuster {
                bounds: vec![150.0, 5191.0, 7450.0],
                rates: vec![1.08, 0.85, 1.16],
            }),
            Rc::new(MinimumAdaptiveApprovalAgreementAdjuster { minimum: 13.4 }),
            Rc::new(PercentAdaptiveApprovalAgreementAdjuster { percent: -3.2 }),
            Rc::new(AdaptiveApprovalAgreementCeilingOffsetAdjuster { weight: 1.55 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 25118.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_approval_agreement_engine();
        let item = new_adaptive_approval_agreement("code-810", "label-252");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
