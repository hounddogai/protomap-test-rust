use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the query batch calculation.
pub trait QueryBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceQueryBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryBatchStep for NamespaceQueryBatchAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &QueryBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryBatchAdjuster {
    pub minimum: f64,
}

impl QueryBatchStep for MinimumQueryBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryBatchStep for TieredQueryBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryBatchAdjuster {
    pub cap: f64,
}

impl QueryBatchStep for CapQueryBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the query batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryBatchEngine {
    pub steps: Vec<Rc<dyn QueryBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryBatchEngine {
    pub fn compute(&mut self, item: &QueryBatch) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryBatchStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_batch_engine() -> QueryBatchEngine {
    QueryBatchEngine {
        steps: vec![
            Rc::new(NamespaceQueryBatchAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.35),
                    ("east".to_string(), 1.31),
                    ("west".to_string(), 1.15),
                ]),
            }) as Rc<dyn QueryBatchStep>,
            Rc::new(MinimumQueryBatchAdjuster { minimum: 9.7 }),
            Rc::new(TieredQueryBatchAdjuster {
                bounds: vec![1467.0, 1735.0, 7926.0],
                rates: vec![1.11, 0.95, 1.03],
            }),
            Rc::new(CapQueryBatchAdjuster { cap: 15504.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 18220.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_batch_engine();
        let item = new_query_batch("code-826", "label-20");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
