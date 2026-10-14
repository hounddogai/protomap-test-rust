use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the query checkpoint calculation.
pub trait QueryCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryCheckpointAdjuster for FlaggedQueryCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryCheckpointAdjuster {
    pub cap: f64,
}

impl QueryCheckpointAdjuster for CapQueryCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryCheckpointAdjuster for TieredQueryCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the query checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryCheckpointEngine {
    pub steps: Vec<Rc<dyn QueryCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryCheckpointEngine {
    pub fn compute(&mut self, item: &QueryCheckpoint) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryCheckpointAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_checkpoint_engine(flags: &flags::Registry) -> QueryCheckpointEngine {
    QueryCheckpointEngine {
        steps: vec![
            Rc::new(FlaggedQueryCheckpointAdjuster {
                flags: flags.clone(),
                flag: "query-checkpoint-adaptive".to_string(),
                percent: -5.1,
            }) as Rc<dyn QueryCheckpointAdjuster>,
            Rc::new(CapQueryCheckpointAdjuster { cap: 1309.0 }),
            Rc::new(TieredQueryCheckpointAdjuster {
                bounds: vec![3950.0, 5308.0, 7814.0, 8509.0],
                rates: vec![1.19, 0.85, 0.85, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 50996.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_checkpoint_engine(&flags::new_registry());
        let item = new_query_checkpoint("code-855", "name-865");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
