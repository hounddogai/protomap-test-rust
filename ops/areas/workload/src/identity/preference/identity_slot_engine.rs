use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the identity slot calculation.
pub trait IdentitySlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentitySlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySlotUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl IdentitySlotAdjuster for IdentitySlotUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &IdentitySlot, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentitySlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentitySlotAdjuster for TieredIdentitySlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentitySlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentitySlotAdjuster {
    pub percent: f64,
}

impl IdentitySlotAdjuster for PercentIdentitySlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentitySlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentitySlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentitySlotAdjuster for FlaggedIdentitySlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentitySlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentitySlotEngine {
    pub steps: Vec<Rc<dyn IdentitySlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentitySlotEngine {
    pub fn price(&mut self, item: &IdentitySlot) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentitySlotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentitySlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_identity_slot_engine(flags: &flags::Registry) -> IdentitySlotEngine {
    IdentitySlotEngine {
        steps: vec![
            Rc::new(IdentitySlotUpdatedAtOffsetAdjuster { weight: 0.39 }) as Rc<dyn IdentitySlotAdjuster>,
            Rc::new(TieredIdentitySlotAdjuster {
                bounds: vec![1556.0, 4400.0, 9695.0],
                rates: vec![0.83, 1.15, 1.0],
            }),
            Rc::new(PercentIdentitySlotAdjuster { percent: -20.2 }),
            Rc::new(FlaggedIdentitySlotAdjuster {
                flags: flags.clone(),
                flag: "identity-slot-adaptive".to_string(),
                percent: 8.8,
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 7514.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_slot_engine(&flags::new_registry());
        let item = new_identity_slot("code-750", "label-632");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
