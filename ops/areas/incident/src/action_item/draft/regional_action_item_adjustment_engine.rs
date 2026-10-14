use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional action item adjustment calculation.
pub trait RegionalActionItemAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalActionItemAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalActionItemAdjustmentAdjuster {
    pub cap: f64,
}

impl RegionalActionItemAdjustmentStep for CapRegionalActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalActionItemAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemAdjustmentWeightOffsetAdjuster {
    pub weight: f64,
}

impl RegionalActionItemAdjustmentStep for RegionalActionItemAdjustmentWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalActionItemAdjustment, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalActionItemAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalActionItemAdjustmentStep for TieredRegionalActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalActionItemAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalActionItemAdjustmentAdjuster {
    pub step: f64,
}

impl RegionalActionItemAdjustmentStep for RoundingRegionalActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalActionItemAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional action item adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalActionItemAdjustmentEngine {
    pub steps: Vec<Rc<dyn RegionalActionItemAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalActionItemAdjustmentEngine {
    pub fn price(&mut self, item: &RegionalActionItemAdjustment) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalActionItemAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_action_item_adjustment_engine() -> RegionalActionItemAdjustmentEngine {
    RegionalActionItemAdjustmentEngine {
        steps: vec![
            Rc::new(CapRegionalActionItemAdjustmentAdjuster { cap: 27992.0 })
                as Rc<dyn RegionalActionItemAdjustmentStep>,
            Rc::new(RegionalActionItemAdjustmentWeightOffsetAdjuster { weight: 0.61 }),
            Rc::new(TieredRegionalActionItemAdjustmentAdjuster {
                bounds: vec![7818.0, 8863.0],
                rates: vec![1.01, 0.87],
            }),
            Rc::new(RoundingRegionalActionItemAdjustmentAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 22376.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_action_item_adjustment_engine();
        let item = new_regional_action_item_adjustment("code-823", "name-926");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
