use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending silence allocation calculation.
pub trait PendingSilenceAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingSilenceAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceAllocationStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl PendingSilenceAllocationModifier for PendingSilenceAllocationStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &PendingSilenceAllocation, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingSilenceAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingSilenceAllocationModifier for FlaggedPendingSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingSilenceAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingSilenceAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingSilenceAllocationModifier for TieredPendingSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingSilenceAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingSilenceAllocationAdjuster {
    pub step: f64,
}

impl PendingSilenceAllocationModifier for RoundingPendingSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingSilenceAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingSilenceAllocationAdjuster {
    pub minimum: f64,
}

impl PendingSilenceAllocationModifier for MinimumPendingSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingSilenceAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending silence allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingSilenceAllocationEngine {
    pub steps: Vec<Rc<dyn PendingSilenceAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingSilenceAllocationEngine {
    pub fn price(&mut self, item: &PendingSilenceAllocation) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingSilenceAllocationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingSilenceAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_silence_allocation_engine(flags: &flags::Registry) -> PendingSilenceAllocationEngine {
    PendingSilenceAllocationEngine {
        steps: vec![
            Rc::new(PendingSilenceAllocationStartsAtOffsetAdjuster { weight: 0.35 })
                as Rc<dyn PendingSilenceAllocationModifier>,
            Rc::new(FlaggedPendingSilenceAllocationAdjuster {
                flags: flags.clone(),
                flag: "pending-silence-allocation-lenient".to_string(),
                percent: -8.0,
            }),
            Rc::new(TieredPendingSilenceAllocationAdjuster {
                bounds: vec![942.0, 5648.0, 9169.0, 9631.0],
                rates: vec![1.08, 1.0, 1.07, 1.04],
            }),
            Rc::new(RoundingPendingSilenceAllocationAdjuster { step: 0.05 }),
            Rc::new(MinimumPendingSilenceAllocationAdjuster { minimum: 45.6 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 37898.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_silence_allocation_engine(&flags::new_registry());
        let item = new_pending_silence_allocation("code-247", "name-417");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
