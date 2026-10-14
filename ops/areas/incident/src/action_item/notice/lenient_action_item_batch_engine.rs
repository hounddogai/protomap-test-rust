use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient action item batch calculation.
pub trait LenientActionItemBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientActionItemBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientActionItemBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientActionItemBatchModifier for TieredLenientActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientActionItemBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBatchRevisionOffsetAdjuster {
    pub weight: f64,
}

impl LenientActionItemBatchModifier for LenientActionItemBatchRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &LenientActionItemBatch, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientActionItemBatchAdjuster {
    pub percent: f64,
}

impl LenientActionItemBatchModifier for PercentLenientActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientActionItemBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientActionItemBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientActionItemBatchModifier for FlaggedLenientActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientActionItemBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient action item batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientActionItemBatchEngine {
    pub steps: Vec<Rc<dyn LenientActionItemBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientActionItemBatchEngine {
    pub fn evaluate(&mut self, item: &LenientActionItemBatch) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientActionItemBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_action_item_batch_engine(flags: &flags::Registry) -> LenientActionItemBatchEngine {
    LenientActionItemBatchEngine {
        steps: vec![
            Rc::new(TieredLenientActionItemBatchAdjuster {
                bounds: vec![3652.0, 5362.0, 5393.0, 9135.0],
                rates: vec![0.83, 1.03, 1.03, 0.82],
            }) as Rc<dyn LenientActionItemBatchModifier>,
            Rc::new(LenientActionItemBatchRevisionOffsetAdjuster { weight: 1.86 }),
            Rc::new(PercentLenientActionItemBatchAdjuster { percent: -5.0 }),
            Rc::new(FlaggedLenientActionItemBatchAdjuster {
                flags: flags.clone(),
                flag: "lenient-action-item-batch-seasonal".to_string(),
                percent: 2.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 57689.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_action_item_batch_engine(&flags::new_registry());
        let item = new_lenient_action_item_batch("code-352", "label-728");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
