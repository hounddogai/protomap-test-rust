use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive measurement agreement calculation.
pub trait AdaptiveMeasurementAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveMeasurementAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveMeasurementAgreementAdjuster {
    pub cap: f64,
}

impl AdaptiveMeasurementAgreementModifier for CapAdaptiveMeasurementAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveMeasurementAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveMeasurementAgreementAdjuster {
    pub minimum: f64,
}

impl AdaptiveMeasurementAgreementModifier for MinimumAdaptiveMeasurementAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveMeasurementAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveMeasurementAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveMeasurementAgreementModifier for FlaggedAdaptiveMeasurementAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveMeasurementAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementAgreementPortOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveMeasurementAgreementModifier for AdaptiveMeasurementAgreementPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveMeasurementAgreement, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveMeasurementAgreementAdjuster {
    pub step: f64,
}

impl AdaptiveMeasurementAgreementModifier for RoundingAdaptiveMeasurementAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveMeasurementAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive measurement agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveMeasurementAgreementEngine {
    pub steps: Vec<Rc<dyn AdaptiveMeasurementAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveMeasurementAgreementEngine {
    pub fn evaluate(&mut self, item: &AdaptiveMeasurementAgreement) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveMeasurementAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_measurement_agreement_engine(flags: &flags::Registry) -> AdaptiveMeasurementAgreementEngine {
    AdaptiveMeasurementAgreementEngine {
        steps: vec![
            Rc::new(CapAdaptiveMeasurementAgreementAdjuster { cap: 24559.0 })
                as Rc<dyn AdaptiveMeasurementAgreementModifier>,
            Rc::new(MinimumAdaptiveMeasurementAgreementAdjuster { minimum: 13.5 }),
            Rc::new(FlaggedAdaptiveMeasurementAgreementAdjuster {
                flags: flags.clone(),
                flag: "adaptive-measurement-agreement-strict".to_string(),
                percent: 8.9,
            }),
            Rc::new(AdaptiveMeasurementAgreementPortOffsetAdjuster { weight: 0.72 }),
            Rc::new(RoundingAdaptiveMeasurementAgreementAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 16086.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_measurement_agreement_engine(&flags::new_registry());
        let item = new_adaptive_measurement_agreement("code-268", "label-999");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
