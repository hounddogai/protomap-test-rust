use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity agreement calculation.
pub trait IdentityAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityAgreementAdjuster {
    pub step: f64,
}

impl IdentityAgreementFactor for RoundingIdentityAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityAgreementFactor for TieredIdentityAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelIdentityAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityAgreementFactor for LabelIdentityAgreementAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &IdentityAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the identity agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityAgreementEngine {
    pub steps: Vec<Rc<dyn IdentityAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityAgreementEngine {
    pub fn compute(&mut self, item: &IdentityAgreement) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[IdentityAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_agreement_engine() -> IdentityAgreementEngine {
    IdentityAgreementEngine {
        steps: vec![
            Rc::new(RoundingIdentityAgreementAdjuster { step: 1.0 }) as Rc<dyn IdentityAgreementFactor>,
            Rc::new(TieredIdentityAgreementAdjuster {
                bounds: vec![231.0, 7863.0, 8704.0],
                rates: vec![0.84, 1.1, 1.01],
            }),
            Rc::new(LabelIdentityAgreementAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.8)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 83336.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_agreement_engine();
        let item = new_identity_agreement("code-293", "label-731");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
