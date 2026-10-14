use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched timeline record calculation.
pub trait BatchedTimelineRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedTimelineRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedTimelineRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedTimelineRecordModifier for NameBatchedTimelineRecordAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedTimelineRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedTimelineRecordAdjuster {
    pub cap: f64,
}

impl BatchedTimelineRecordModifier for CapBatchedTimelineRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedTimelineRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecordBudgetOffsetAdjuster {
    pub weight: f64,
}

impl BatchedTimelineRecordModifier for BatchedTimelineRecordBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedTimelineRecord, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedTimelineRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedTimelineRecordModifier for TieredBatchedTimelineRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedTimelineRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched timeline record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedTimelineRecordEngine {
    pub steps: Vec<Rc<dyn BatchedTimelineRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedTimelineRecordEngine {
    pub fn compute(&mut self, item: &BatchedTimelineRecord) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedTimelineRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedTimelineRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_timeline_record_engine() -> BatchedTimelineRecordEngine {
    BatchedTimelineRecordEngine {
        steps: vec![
            Rc::new(NameBatchedTimelineRecordAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.32),
                    ("online".to_string(), 1.21),
                    ("south".to_string(), 0.81),
                ]),
            }) as Rc<dyn BatchedTimelineRecordModifier>,
            Rc::new(CapBatchedTimelineRecordAdjuster { cap: 20196.0 }),
            Rc::new(BatchedTimelineRecordBudgetOffsetAdjuster { weight: 0.65 }),
            Rc::new(TieredBatchedTimelineRecordAdjuster {
                bounds: vec![1865.0, 3333.0, 6809.0, 8829.0],
                rates: vec![0.95, 0.98, 1.04, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 44245.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_timeline_record_engine();
        let item = new_batched_timeline_record("code-43", "name-457");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
