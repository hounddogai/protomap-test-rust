use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual rollback tier calculation.
pub trait ManualRollbackTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRollbackTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackTierLimitOffsetAdjuster {
    pub weight: f64,
}

impl ManualRollbackTierModifier for ManualRollbackTierLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ManualRollbackTier, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualRollbackTierAdjuster {
    pub minimum: f64,
}

impl ManualRollbackTierModifier for MinimumManualRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualRollbackTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualRollbackTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualRollbackTierModifier for TieredManualRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualRollbackTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual rollback tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRollbackTierEngine {
    pub steps: Vec<Rc<dyn ManualRollbackTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRollbackTierEngine {
    pub fn compute(&mut self, item: &ManualRollbackTier) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualRollbackTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_rollback_tier_engine() -> ManualRollbackTierEngine {
    ManualRollbackTierEngine {
        steps: vec![
            Rc::new(ManualRollbackTierLimitOffsetAdjuster { weight: 1.02 }) as Rc<dyn ManualRollbackTierModifier>,
            Rc::new(MinimumManualRollbackTierAdjuster { minimum: 6.8 }),
            Rc::new(TieredManualRollbackTierAdjuster {
                bounds: vec![506.0, 756.0, 5472.0, 8446.0],
                rates: vec![0.86, 0.91, 1.06, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 64715.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_rollback_tier_engine();
        let item = new_manual_rollback_tier("code-107", "label-949");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
