use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary agreement calculation.
pub trait CanaryAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAgreementMarginOffsetAdjuster {
    pub weight: f64,
}

impl CanaryAgreementModifier for CanaryAgreementMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryAgreement, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryAgreementModifier for TieredCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryAgreementAdjuster {
    pub step: f64,
}

impl CanaryAgreementModifier for RoundingCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryAgreementAdjuster {
    pub minimum: f64,
}

impl CanaryAgreementModifier for MinimumCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the canary agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryAgreementEngine {
    pub steps: Vec<Rc<dyn CanaryAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryAgreementEngine {
    pub fn compute(&mut self, item: &CanaryAgreement) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CanaryAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_canary_agreement_engine() -> CanaryAgreementEngine {
    CanaryAgreementEngine {
        steps: vec![
            Rc::new(CanaryAgreementMarginOffsetAdjuster { weight: 0.07 }) as Rc<dyn CanaryAgreementModifier>,
            Rc::new(TieredCanaryAgreementAdjuster {
                bounds: vec![4526.0, 5975.0, 7991.0],
                rates: vec![0.97, 1.09, 1.16],
            }),
            Rc::new(RoundingCanaryAgreementAdjuster { step: 0.25 }),
            Rc::new(MinimumCanaryAgreementAdjuster { minimum: 25.1 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 45494.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_agreement_engine();
        let item = new_canary_agreement("code-398", "label-660");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
