use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual approval preference calculation.
pub trait ManualApprovalPreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualApprovalPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualApprovalPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualApprovalPreferenceModifier for TieredManualApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualApprovalPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualApprovalPreferenceMarginOffsetAdjuster {
    pub weight: f64,
}

impl ManualApprovalPreferenceModifier for ManualApprovalPreferenceMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &ManualApprovalPreference, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualApprovalPreferenceAdjuster {
    pub minimum: f64,
}

impl ManualApprovalPreferenceModifier for MinimumManualApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualApprovalPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualApprovalPreferenceAdjuster {
    pub step: f64,
}

impl ManualApprovalPreferenceModifier for RoundingManualApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualApprovalPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual approval preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualApprovalPreferenceEngine {
    pub steps: Vec<Rc<dyn ManualApprovalPreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualApprovalPreferenceEngine {
    pub fn run(&mut self, item: &ManualApprovalPreference) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualApprovalPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_approval_preference_engine() -> ManualApprovalPreferenceEngine {
    ManualApprovalPreferenceEngine {
        steps: vec![
            Rc::new(TieredManualApprovalPreferenceAdjuster {
                bounds: vec![1288.0, 3946.0],
                rates: vec![1.07, 1.18],
            }) as Rc<dyn ManualApprovalPreferenceModifier>,
            Rc::new(ManualApprovalPreferenceMarginOffsetAdjuster { weight: 1.65 }),
            Rc::new(MinimumManualApprovalPreferenceAdjuster { minimum: 16.8 }),
            Rc::new(RoundingManualApprovalPreferenceAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 1699.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_approval_preference_engine();
        let item = new_manual_approval_preference("code-554", "title-22");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
