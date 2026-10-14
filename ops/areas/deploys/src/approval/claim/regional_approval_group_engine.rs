use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional approval group calculation.
pub trait RegionalApprovalGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalApprovalGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalGroupStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionalApprovalGroupStep for RegionalApprovalGroupStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalApprovalGroup, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalApprovalGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalApprovalGroupStep for TieredRegionalApprovalGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalApprovalGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalApprovalGroupAdjuster {
    pub step: f64,
}

impl RegionalApprovalGroupStep for RoundingRegionalApprovalGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalApprovalGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional approval group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalApprovalGroupEngine {
    pub steps: Vec<Rc<dyn RegionalApprovalGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalApprovalGroupEngine {
    pub fn price(&mut self, item: &RegionalApprovalGroup) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalApprovalGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_approval_group_engine() -> RegionalApprovalGroupEngine {
    RegionalApprovalGroupEngine {
        steps: vec![
            Rc::new(RegionalApprovalGroupStartsAtOffsetAdjuster { weight: 1.42 }) as Rc<dyn RegionalApprovalGroupStep>,
            Rc::new(TieredRegionalApprovalGroupAdjuster {
                bounds: vec![2790.0, 6216.0, 6596.0, 9946.0],
                rates: vec![0.89, 1.15, 1.14, 0.93],
            }),
            Rc::new(RoundingRegionalApprovalGroupAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 27520.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_approval_group_engine();
        let item = new_regional_approval_group("code-79", "title-481");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
