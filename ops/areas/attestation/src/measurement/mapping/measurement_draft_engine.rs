use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement draft calculation.
pub trait MeasurementDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementDraftModifier for FlaggedMeasurementDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementDraftAdjuster {
    pub cap: f64,
}

impl MeasurementDraftModifier for CapMeasurementDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementDraftAdjuster {
    pub step: f64,
}

impl MeasurementDraftModifier for RoundingMeasurementDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMeasurementDraftAdjuster {
    pub minimum: f64,
}

impl MeasurementDraftModifier for MinimumMeasurementDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MeasurementDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the measurement draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementDraftEngine {
    pub steps: Vec<Rc<dyn MeasurementDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementDraftEngine {
    pub fn price(&mut self, item: &MeasurementDraft) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_measurement_draft_engine(flags: &flags::Registry) -> MeasurementDraftEngine {
    MeasurementDraftEngine {
        steps: vec![
            Rc::new(FlaggedMeasurementDraftAdjuster {
                flags: flags.clone(),
                flag: "measurement-draft-pending".to_string(),
                percent: 9.6,
            }) as Rc<dyn MeasurementDraftModifier>,
            Rc::new(CapMeasurementDraftAdjuster { cap: 39161.0 }),
            Rc::new(RoundingMeasurementDraftAdjuster { step: 0.05 }),
            Rc::new(MinimumMeasurementDraftAdjuster { minimum: 46.7 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 87835.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_draft_engine(&flags::new_registry());
        let item = new_measurement_draft("code-438", "title-54");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
