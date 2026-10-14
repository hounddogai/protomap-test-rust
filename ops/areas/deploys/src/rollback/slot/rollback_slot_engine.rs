use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback slot calculation.
pub trait RollbackSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackSlotAdjuster {
    pub cap: f64,
}

impl RollbackSlotModifier for CapRollbackSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackSlotAdjuster {
    pub step: f64,
}

impl RollbackSlotModifier for RoundingRollbackSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackSlotModifier for FlaggedRollbackSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRollbackSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackSlotModifier for UnitRollbackSlotAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RollbackSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackSlotAdjuster {
    pub minimum: f64,
}

impl RollbackSlotModifier for MinimumRollbackSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackSlotModifier for TieredRollbackSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rollback slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackSlotEngine {
    pub steps: Vec<Rc<dyn RollbackSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackSlotEngine {
    pub fn compute(&mut self, item: &RollbackSlot) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackSlotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_slot_engine(flags: &flags::Registry) -> RollbackSlotEngine {
    RollbackSlotEngine {
        steps: vec![
            Rc::new(CapRollbackSlotAdjuster { cap: 33382.0 }) as Rc<dyn RollbackSlotModifier>,
            Rc::new(RoundingRollbackSlotAdjuster { step: 5.0 }),
            Rc::new(FlaggedRollbackSlotAdjuster {
                flags: flags.clone(),
                flag: "rollback-slot-regional".to_string(),
                percent: -7.1,
            }),
            Rc::new(UnitRollbackSlotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.18)]),
            }),
            Rc::new(MinimumRollbackSlotAdjuster { minimum: 45.0 }),
            Rc::new(TieredRollbackSlotAdjuster {
                bounds: vec![752.0, 833.0, 3512.0],
                rates: vec![0.86, 1.08, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 95029.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_slot_engine(&flags::new_registry());
        let item = new_rollback_slot("code-513", "label-100");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
