use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary escalation slot calculation.
pub trait PrimaryEscalationSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEscalationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryEscalationSlotAdjuster {
    pub cap: f64,
}

impl PrimaryEscalationSlotFactor for CapPrimaryEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryEscalationSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryEscalationSlotAdjuster {
    pub percent: f64,
}

impl PrimaryEscalationSlotFactor for PercentPrimaryEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryEscalationSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationSlotRetriesOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryEscalationSlotFactor for PrimaryEscalationSlotRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryEscalationSlot, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryEscalationSlotAdjuster {
    pub minimum: f64,
}

impl PrimaryEscalationSlotFactor for MinimumPrimaryEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryEscalationSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPrimaryEscalationSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryEscalationSlotFactor for ClusterPrimaryEscalationSlotAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PrimaryEscalationSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryEscalationSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryEscalationSlotFactor for TieredPrimaryEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryEscalationSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary escalation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationSlotEngine {
    pub steps: Vec<Rc<dyn PrimaryEscalationSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEscalationSlotEngine {
    pub fn compute(&mut self, item: &PrimaryEscalationSlot) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEscalationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_escalation_slot_engine() -> PrimaryEscalationSlotEngine {
    PrimaryEscalationSlotEngine {
        steps: vec![
            Rc::new(CapPrimaryEscalationSlotAdjuster { cap: 32964.0 }) as Rc<dyn PrimaryEscalationSlotFactor>,
            Rc::new(PercentPrimaryEscalationSlotAdjuster { percent: -4.1 }),
            Rc::new(PrimaryEscalationSlotRetriesOffsetAdjuster { weight: 1.94 }),
            Rc::new(MinimumPrimaryEscalationSlotAdjuster { minimum: 24.1 }),
            Rc::new(ClusterPrimaryEscalationSlotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.36), ("central".to_string(), 0.75)]),
            }),
            Rc::new(TieredPrimaryEscalationSlotAdjuster {
                bounds: vec![1717.0, 3791.0, 4853.0, 9772.0],
                rates: vec![1.05, 0.84, 1.06, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 53554.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_escalation_slot_engine();
        let item = new_primary_escalation_slot("code-339", "name-433");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
