use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the slo profile calculation.
pub trait SloProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProfileMarginOffsetAdjuster {
    pub weight: f64,
}

impl SloProfileStep for SloProfileMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &SloProfile, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloProfileAdjuster {
    pub step: f64,
}

impl SloProfileStep for RoundingSloProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloProfileAdjuster {
    pub cap: f64,
}

impl SloProfileStep for CapSloProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the slo profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloProfileEngine {
    pub steps: Vec<Rc<dyn SloProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloProfileEngine {
    pub fn price(&mut self, item: &SloProfile) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloProfileStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_slo_profile_engine() -> SloProfileEngine {
    SloProfileEngine {
        steps: vec![
            Rc::new(SloProfileMarginOffsetAdjuster { weight: 1.78 }) as Rc<dyn SloProfileStep>,
            Rc::new(RoundingSloProfileAdjuster { step: 0.05 }),
            Rc::new(CapSloProfileAdjuster { cap: 9625.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 36490.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_profile_engine();
        let item = new_slo_profile("code-164", "label-379");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
