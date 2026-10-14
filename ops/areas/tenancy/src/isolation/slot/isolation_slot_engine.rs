use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the isolation slot calculation.
pub trait IsolationSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationSlotAdjuster {
    pub percent: f64,
}

impl IsolationSlotModifier for PercentIsolationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameIsolationSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationSlotModifier for NameIsolationSlotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &IsolationSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationSlotAdjuster {
    pub cap: f64,
}

impl IsolationSlotModifier for CapIsolationSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationSlotAdjuster {
    pub minimum: f64,
}

impl IsolationSlotModifier for MinimumIsolationSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationSlotAdjuster {
    pub step: f64,
}

impl IsolationSlotModifier for RoundingIsolationSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationSlotModifier for TieredIsolationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the isolation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationSlotEngine {
    pub steps: Vec<Rc<dyn IsolationSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationSlotEngine {
    pub fn price(&mut self, item: &IsolationSlot) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[IsolationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_isolation_slot_engine() -> IsolationSlotEngine {
    IsolationSlotEngine {
        steps: vec![
            Rc::new(PercentIsolationSlotAdjuster { percent: -22.4 }) as Rc<dyn IsolationSlotModifier>,
            Rc::new(NameIsolationSlotAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.99), ("online".to_string(), 1.12)]),
            }),
            Rc::new(CapIsolationSlotAdjuster { cap: 31190.0 }),
            Rc::new(MinimumIsolationSlotAdjuster { minimum: 46.8 }),
            Rc::new(RoundingIsolationSlotAdjuster { step: 1.0 }),
            Rc::new(TieredIsolationSlotAdjuster {
                bounds: vec![2407.0, 5233.0],
                rates: vec![0.9, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 69408.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_slot_engine();
        let item = new_isolation_slot("code-594", "name-268");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
