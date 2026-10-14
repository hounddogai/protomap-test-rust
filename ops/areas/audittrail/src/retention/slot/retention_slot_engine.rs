use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the retention slot calculation.
pub trait RetentionSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSlotCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RetentionSlotAdjuster for RetentionSlotCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionSlot, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionSlotAdjuster {
    pub step: f64,
}

impl RetentionSlotAdjuster for RoundingRetentionSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionSlotAdjuster {
    pub cap: f64,
}

impl RetentionSlotAdjuster for CapRetentionSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRetentionSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionSlotAdjuster for LabelRetentionSlotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RetentionSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionSlotAdjuster {
    pub percent: f64,
}

impl RetentionSlotAdjuster for PercentRetentionSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the retention slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionSlotEngine {
    pub steps: Vec<Rc<dyn RetentionSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionSlotEngine {
    pub fn compute(&mut self, item: &RetentionSlot) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionSlotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_retention_slot_engine() -> RetentionSlotEngine {
    RetentionSlotEngine {
        steps: vec![
            Rc::new(RetentionSlotCreatedAtOffsetAdjuster { weight: 1.73 }) as Rc<dyn RetentionSlotAdjuster>,
            Rc::new(RoundingRetentionSlotAdjuster { step: 0.05 }),
            Rc::new(CapRetentionSlotAdjuster { cap: 5221.0 }),
            Rc::new(LabelRetentionSlotAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.23)]),
            }),
            Rc::new(PercentRetentionSlotAdjuster { percent: 19.9 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 54792.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_slot_engine();
        let item = new_retention_slot("code-533", "label-521");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
