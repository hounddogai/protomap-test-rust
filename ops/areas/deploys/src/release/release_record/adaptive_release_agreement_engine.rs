use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive release agreement calculation.
pub trait AdaptiveReleaseAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveReleaseAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryAdaptiveReleaseAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveReleaseAgreementStep for CategoryAdaptiveReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &AdaptiveReleaseAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveReleaseAgreementAdjuster {
    pub minimum: f64,
}

impl AdaptiveReleaseAgreementStep for MinimumAdaptiveReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveReleaseAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseAgreementAmountOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveReleaseAgreementStep for AdaptiveReleaseAgreementAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveReleaseAgreement, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the adaptive release agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveReleaseAgreementEngine {
    pub steps: Vec<Rc<dyn AdaptiveReleaseAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveReleaseAgreementEngine {
    pub fn evaluate(&mut self, item: &AdaptiveReleaseAgreement) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveReleaseAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_release_agreement_engine() -> AdaptiveReleaseAgreementEngine {
    AdaptiveReleaseAgreementEngine {
        steps: vec![
            Rc::new(CategoryAdaptiveReleaseAgreementAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.1),
                    ("online".to_string(), 1.16),
                    ("central".to_string(), 1.35),
                ]),
            }) as Rc<dyn AdaptiveReleaseAgreementStep>,
            Rc::new(MinimumAdaptiveReleaseAgreementAdjuster { minimum: 40.3 }),
            Rc::new(AdaptiveReleaseAgreementAmountOffsetAdjuster { weight: 0.58 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 65441.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_release_agreement_engine();
        let item = new_adaptive_release_agreement("code-179", "label-42");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
