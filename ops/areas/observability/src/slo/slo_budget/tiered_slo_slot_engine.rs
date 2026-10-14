use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered slo slot calculation.
pub trait TieredSloSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredSloSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredSloSlotAdjuster {
    pub minimum: f64,
}

impl TieredSloSlotStep for MinimumTieredSloSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredSloSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTieredSloSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredSloSlotStep for LabelTieredSloSlotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TieredSloSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloSlotThresholdOffsetAdjuster {
    pub weight: f64,
}

impl TieredSloSlotStep for TieredSloSlotThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &TieredSloSlot, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the tiered slo slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredSloSlotEngine {
    pub steps: Vec<Rc<dyn TieredSloSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredSloSlotEngine {
    pub fn evaluate(&mut self, item: &TieredSloSlot) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredSloSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredSloSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_slo_slot_engine() -> TieredSloSlotEngine {
    TieredSloSlotEngine {
        steps: vec![
            Rc::new(MinimumTieredSloSlotAdjuster { minimum: 29.4 }) as Rc<dyn TieredSloSlotStep>,
            Rc::new(LabelTieredSloSlotAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.21),
                    ("south".to_string(), 1.03),
                    ("west".to_string(), 0.84),
                ]),
            }),
            Rc::new(TieredSloSlotThresholdOffsetAdjuster { weight: 1.97 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 39423.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_slo_slot_engine();
        let item = new_tiered_slo_slot("code-307", "label-838");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
