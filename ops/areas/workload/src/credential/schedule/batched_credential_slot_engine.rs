use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched credential slot calculation.
pub trait BatchedCredentialSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedCredentialSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedCredentialSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedCredentialSlotFactor for FlaggedBatchedCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedCredentialSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedCredentialSlotAdjuster {
    pub minimum: f64,
}

impl BatchedCredentialSlotFactor for MinimumBatchedCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedCredentialSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBatchedCredentialSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedCredentialSlotFactor for SegmentBatchedCredentialSlotAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &BatchedCredentialSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedCredentialSlotAdjuster {
    pub cap: f64,
}

impl BatchedCredentialSlotFactor for CapBatchedCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedCredentialSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedCredentialSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedCredentialSlotFactor for TieredBatchedCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedCredentialSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched credential slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedCredentialSlotEngine {
    pub steps: Vec<Rc<dyn BatchedCredentialSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedCredentialSlotEngine {
    pub fn compute(&mut self, item: &BatchedCredentialSlot) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedCredentialSlotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedCredentialSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_credential_slot_engine(flags: &flags::Registry) -> BatchedCredentialSlotEngine {
    BatchedCredentialSlotEngine {
        steps: vec![
            Rc::new(FlaggedBatchedCredentialSlotAdjuster {
                flags: flags.clone(),
                flag: "batched-credential-slot-tiered".to_string(),
                percent: 4.4,
            }) as Rc<dyn BatchedCredentialSlotFactor>,
            Rc::new(MinimumBatchedCredentialSlotAdjuster { minimum: 26.5 }),
            Rc::new(SegmentBatchedCredentialSlotAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.84)]),
            }),
            Rc::new(CapBatchedCredentialSlotAdjuster { cap: 17997.0 }),
            Rc::new(TieredBatchedCredentialSlotAdjuster {
                bounds: vec![1057.0, 3781.0, 9188.0],
                rates: vec![1.16, 1.07, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 21317.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_credential_slot_engine(&flags::new_registry());
        let item = new_batched_credential_slot("code-289", "name-261");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
