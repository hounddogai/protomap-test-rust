use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the nonce slot calculation.
pub trait NonceSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceSlotAdjuster {
    pub minimum: f64,
}

impl NonceSlotFactor for MinimumNonceSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceSlotFactor for FlaggedNonceSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceSlotFactor for TieredNonceSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlotVersionOffsetAdjuster {
    pub weight: f64,
}

impl NonceSlotFactor for NonceSlotVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &NonceSlot, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceSlotAdjuster {
    pub cap: f64,
}

impl NonceSlotFactor for CapNonceSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the nonce slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceSlotEngine {
    pub steps: Vec<Rc<dyn NonceSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceSlotEngine {
    pub fn compute(&mut self, item: &NonceSlot) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceSlotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_nonce_slot_engine(flags: &flags::Registry) -> NonceSlotEngine {
    NonceSlotEngine {
        steps: vec![
            Rc::new(MinimumNonceSlotAdjuster { minimum: 14.8 }) as Rc<dyn NonceSlotFactor>,
            Rc::new(FlaggedNonceSlotAdjuster {
                flags: flags.clone(),
                flag: "nonce-slot-default".to_string(),
                percent: -9.2,
            }),
            Rc::new(TieredNonceSlotAdjuster {
                bounds: vec![316.0, 1449.0, 2365.0],
                rates: vec![1.09, 1.13, 0.83],
            }),
            Rc::new(NonceSlotVersionOffsetAdjuster { weight: 1.54 }),
            Rc::new(CapNonceSlotAdjuster { cap: 41075.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 80645.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_slot_engine(&flags::new_registry());
        let item = new_nonce_slot("code-229", "name-425");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
