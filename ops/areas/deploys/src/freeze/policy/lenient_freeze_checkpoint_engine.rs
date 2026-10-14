use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient freeze checkpoint calculation.
pub trait LenientFreezeCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientFreezeCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientFreezeCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientFreezeCheckpointFactor for FlaggedLenientFreezeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientFreezeCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientFreezeCheckpointAdjuster {
    pub step: f64,
}

impl LenientFreezeCheckpointFactor for RoundingLenientFreezeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientFreezeCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpointAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl LenientFreezeCheckpointFactor for LenientFreezeCheckpointAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &LenientFreezeCheckpoint, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientFreezeCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientFreezeCheckpointFactor for TieredLenientFreezeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientFreezeCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceLenientFreezeCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientFreezeCheckpointFactor for NamespaceLenientFreezeCheckpointAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &LenientFreezeCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient freeze checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientFreezeCheckpointEngine {
    pub steps: Vec<Rc<dyn LenientFreezeCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientFreezeCheckpointEngine {
    pub fn run(&mut self, item: &LenientFreezeCheckpoint) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientFreezeCheckpointFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientFreezeCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_freeze_checkpoint_engine(flags: &flags::Registry) -> LenientFreezeCheckpointEngine {
    LenientFreezeCheckpointEngine {
        steps: vec![
            Rc::new(FlaggedLenientFreezeCheckpointAdjuster {
                flags: flags.clone(),
                flag: "lenient-freeze-checkpoint-capped".to_string(),
                percent: -8.4,
            }) as Rc<dyn LenientFreezeCheckpointFactor>,
            Rc::new(RoundingLenientFreezeCheckpointAdjuster { step: 0.1 }),
            Rc::new(LenientFreezeCheckpointAttemptsOffsetAdjuster { weight: 1.4 }),
            Rc::new(TieredLenientFreezeCheckpointAdjuster {
                bounds: vec![1833.0, 6275.0],
                rates: vec![1.05, 0.84],
            }),
            Rc::new(NamespaceLenientFreezeCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.25),
                    ("online".to_string(), 1.35),
                    ("central".to_string(), 0.85),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 51546.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_freeze_checkpoint_engine(&flags::new_registry());
        let item = new_lenient_freeze_checkpoint("code-914", "name-783");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
