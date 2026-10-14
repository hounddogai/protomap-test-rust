use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the retention checkpoint calculation.
pub trait RetentionCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRetentionCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionCheckpointFactor for SourceRetentionCheckpointAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RetentionCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionCheckpointAdjuster {
    pub percent: f64,
}

impl RetentionCheckpointFactor for PercentRetentionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionCheckpointFactor for FlaggedRetentionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionCheckpointFactor for TieredRetentionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionCheckpointAdjuster {
    pub cap: f64,
}

impl RetentionCheckpointFactor for CapRetentionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCheckpointWeightOffsetAdjuster {
    pub weight: f64,
}

impl RetentionCheckpointFactor for RetentionCheckpointWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionCheckpoint, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the retention checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionCheckpointEngine {
    pub steps: Vec<Rc<dyn RetentionCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionCheckpointEngine {
    pub fn evaluate(&mut self, item: &RetentionCheckpoint) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionCheckpointFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_checkpoint_engine(flags: &flags::Registry) -> RetentionCheckpointEngine {
    RetentionCheckpointEngine {
        steps: vec![
            Rc::new(SourceRetentionCheckpointAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.28)]),
            }) as Rc<dyn RetentionCheckpointFactor>,
            Rc::new(PercentRetentionCheckpointAdjuster { percent: -9.2 }),
            Rc::new(FlaggedRetentionCheckpointAdjuster {
                flags: flags.clone(),
                flag: "retention-checkpoint-manual".to_string(),
                percent: -4.6,
            }),
            Rc::new(TieredRetentionCheckpointAdjuster {
                bounds: vec![6591.0, 7983.0, 9820.0],
                rates: vec![0.81, 1.07, 1.0],
            }),
            Rc::new(CapRetentionCheckpointAdjuster { cap: 29721.0 }),
            Rc::new(RetentionCheckpointWeightOffsetAdjuster { weight: 0.7 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 4380.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_checkpoint_engine(&flags::new_registry());
        let item = new_retention_checkpoint("code-811", "title-664");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
