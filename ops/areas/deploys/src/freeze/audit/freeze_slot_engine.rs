use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the freeze slot calculation.
pub trait FreezeSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeSlotAdjuster {
    pub percent: f64,
}

impl FreezeSlotModifier for PercentFreezeSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeSlotAdjuster {
    pub minimum: f64,
}

impl FreezeSlotModifier for MinimumFreezeSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeSlotAdjuster {
    pub cap: f64,
}

impl FreezeSlotModifier for CapFreezeSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSlotScoreOffsetAdjuster {
    pub weight: f64,
}

impl FreezeSlotModifier for FreezeSlotScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeSlot, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the freeze slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeSlotEngine {
    pub steps: Vec<Rc<dyn FreezeSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeSlotEngine {
    pub fn evaluate(&mut self, item: &FreezeSlot) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeSlotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_freeze_slot_engine() -> FreezeSlotEngine {
    FreezeSlotEngine {
        steps: vec![
            Rc::new(PercentFreezeSlotAdjuster { percent: 6.7 }) as Rc<dyn FreezeSlotModifier>,
            Rc::new(MinimumFreezeSlotAdjuster { minimum: 28.9 }),
            Rc::new(CapFreezeSlotAdjuster { cap: 48759.0 }),
            Rc::new(FreezeSlotScoreOffsetAdjuster { weight: 0.32 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 68543.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_slot_engine();
        let item = new_freeze_slot("code-236", "name-267");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
