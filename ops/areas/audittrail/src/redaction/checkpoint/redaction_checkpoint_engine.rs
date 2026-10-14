use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the redaction checkpoint calculation.
pub trait RedactionCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionCheckpointStep for TieredRedactionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionCheckpointAdjuster {
    pub percent: f64,
}

impl RedactionCheckpointStep for PercentRedactionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionCheckpointAdjuster {
    pub minimum: f64,
}

impl RedactionCheckpointStep for MinimumRedactionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpointBurstOffsetAdjuster {
    pub weight: f64,
}

impl RedactionCheckpointStep for RedactionCheckpointBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionCheckpoint, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionCheckpointStep for FlaggedRedactionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the redaction checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionCheckpointEngine {
    pub steps: Vec<Rc<dyn RedactionCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionCheckpointEngine {
    pub fn compute(&mut self, item: &RedactionCheckpoint) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionCheckpointStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_checkpoint_engine(flags: &flags::Registry) -> RedactionCheckpointEngine {
    RedactionCheckpointEngine {
        steps: vec![
            Rc::new(TieredRedactionCheckpointAdjuster {
                bounds: vec![4199.0, 6000.0, 7510.0],
                rates: vec![0.89, 1.03, 1.09],
            }) as Rc<dyn RedactionCheckpointStep>,
            Rc::new(PercentRedactionCheckpointAdjuster { percent: 2.8 }),
            Rc::new(MinimumRedactionCheckpointAdjuster { minimum: 48.5 }),
            Rc::new(RedactionCheckpointBurstOffsetAdjuster { weight: 1.28 }),
            Rc::new(FlaggedRedactionCheckpointAdjuster {
                flags: flags.clone(),
                flag: "redaction-checkpoint-pending".to_string(),
                percent: 7.8,
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 87993.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_checkpoint_engine(&flags::new_registry());
        let item = new_redaction_checkpoint("code-205", "label-371");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
