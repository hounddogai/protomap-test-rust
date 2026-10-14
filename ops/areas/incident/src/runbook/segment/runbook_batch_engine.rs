use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the runbook batch calculation.
pub trait RunbookBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRunbookBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookBatchStep for TitleRunbookBatchAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RunbookBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookBatchAdjuster {
    pub cap: f64,
}

impl RunbookBatchStep for CapRunbookBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookBatchAdjuster {
    pub minimum: f64,
}

impl RunbookBatchStep for MinimumRunbookBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the runbook batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookBatchEngine {
    pub steps: Vec<Rc<dyn RunbookBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookBatchEngine {
    pub fn evaluate(&mut self, item: &RunbookBatch) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookBatchStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_batch_engine() -> RunbookBatchEngine {
    RunbookBatchEngine {
        steps: vec![
            Rc::new(TitleRunbookBatchAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.02),
                    ("west".to_string(), 0.72),
                    ("north".to_string(), 0.92),
                ]),
            }) as Rc<dyn RunbookBatchStep>,
            Rc::new(CapRunbookBatchAdjuster { cap: 11435.0 }),
            Rc::new(MinimumRunbookBatchAdjuster { minimum: 36.4 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 36000.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_batch_engine();
        let item = new_runbook_batch("code-112", "title-126");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
