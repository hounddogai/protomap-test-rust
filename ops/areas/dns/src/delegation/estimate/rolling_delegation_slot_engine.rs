use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling delegation slot calculation.
pub trait RollingDelegationSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingDelegationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingDelegationSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingDelegationSlotStep for TieredRollingDelegationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingDelegationSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlotQuantityOffsetAdjuster {
    pub weight: f64,
}

impl RollingDelegationSlotStep for RollingDelegationSlotQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &RollingDelegationSlot, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRollingDelegationSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingDelegationSlotStep for NamespaceRollingDelegationSlotAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RollingDelegationSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling delegation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingDelegationSlotEngine {
    pub steps: Vec<Rc<dyn RollingDelegationSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingDelegationSlotEngine {
    pub fn price(&mut self, item: &RollingDelegationSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingDelegationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_delegation_slot_engine() -> RollingDelegationSlotEngine {
    RollingDelegationSlotEngine {
        steps: vec![
            Rc::new(TieredRollingDelegationSlotAdjuster {
                bounds: vec![3152.0, 4501.0, 7861.0],
                rates: vec![0.85, 0.94, 1.1],
            }) as Rc<dyn RollingDelegationSlotStep>,
            Rc::new(RollingDelegationSlotQuantityOffsetAdjuster { weight: 1.06 }),
            Rc::new(NamespaceRollingDelegationSlotAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.37),
                    ("east".to_string(), 0.83),
                    ("north".to_string(), 0.93),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 10480.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_delegation_slot_engine();
        let item = new_rolling_delegation_slot("code-330", "name-394");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
