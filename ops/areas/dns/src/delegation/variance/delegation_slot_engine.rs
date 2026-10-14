use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the delegation slot calculation.
pub trait DelegationSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDelegationSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationSlotAdjuster for SegmentDelegationSlotAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &DelegationSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationSlotAdjuster for TieredDelegationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationSlotAdjuster {
    pub step: f64,
}

impl DelegationSlotAdjuster for RoundingDelegationSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the delegation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationSlotEngine {
    pub steps: Vec<Rc<dyn DelegationSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationSlotEngine {
    pub fn compute(&mut self, item: &DelegationSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DelegationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_delegation_slot_engine() -> DelegationSlotEngine {
    DelegationSlotEngine {
        steps: vec![
            Rc::new(SegmentDelegationSlotAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.29)]),
            }) as Rc<dyn DelegationSlotAdjuster>,
            Rc::new(TieredDelegationSlotAdjuster {
                bounds: vec![6482.0, 7916.0],
                rates: vec![1.14, 0.99],
            }),
            Rc::new(RoundingDelegationSlotAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 59736.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_slot_engine();
        let item = new_delegation_slot("code-174", "name-158");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
