use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rolling egress entry calculation.
pub trait RollingEgressEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingEgressEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingEgressEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingEgressEntryModifier for FlaggedRollingEgressEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingEgressEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntryFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RollingEgressEntryModifier for RollingEgressEntryFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RollingEgressEntry, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingEgressEntryAdjuster {
    pub cap: f64,
}

impl RollingEgressEntryModifier for CapRollingEgressEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingEgressEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling egress entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingEgressEntryEngine {
    pub steps: Vec<Rc<dyn RollingEgressEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingEgressEntryEngine {
    pub fn price(&mut self, item: &RollingEgressEntry) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingEgressEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingEgressEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_egress_entry_engine(flags: &flags::Registry) -> RollingEgressEntryEngine {
    RollingEgressEntryEngine {
        steps: vec![
            Rc::new(FlaggedRollingEgressEntryAdjuster {
                flags: flags.clone(),
                flag: "rolling-egress-entry-capped".to_string(),
                percent: 0.0,
            }) as Rc<dyn RollingEgressEntryModifier>,
            Rc::new(RollingEgressEntryFloorValueOffsetAdjuster { weight: 0.9 }),
            Rc::new(CapRollingEgressEntryAdjuster { cap: 9435.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 35829.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_egress_entry_engine(&flags::new_registry());
        let item = new_rolling_egress_entry("code-129", "label-972");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
