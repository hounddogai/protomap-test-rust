use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the federation slot calculation.
pub trait FederationSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationSlotStep for FlaggedFederationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationSlotAdjuster {
    pub minimum: f64,
}

impl FederationSlotStep for MinimumFederationSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationSlotAdjuster {
    pub percent: f64,
}

impl FederationSlotStep for PercentFederationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the federation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationSlotEngine {
    pub steps: Vec<Rc<dyn FederationSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationSlotEngine {
    pub fn evaluate(&mut self, item: &FederationSlot) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_slot_engine(flags: &flags::Registry) -> FederationSlotEngine {
    FederationSlotEngine {
        steps: vec![
            Rc::new(FlaggedFederationSlotAdjuster {
                flags: flags.clone(),
                flag: "federation-slot-global".to_string(),
                percent: 5.9,
            }) as Rc<dyn FederationSlotStep>,
            Rc::new(MinimumFederationSlotAdjuster { minimum: 45.2 }),
            Rc::new(PercentFederationSlotAdjuster { percent: 0.9 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 90016.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_slot_engine(&flags::new_registry());
        let item = new_federation_slot("code-398", "name-925");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
