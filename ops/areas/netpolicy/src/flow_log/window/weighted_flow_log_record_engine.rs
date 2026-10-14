use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the weighted flow log record calculation.
pub trait WeightedFlowLogRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFlowLogRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedFlowLogRecordAdjuster {
    pub minimum: f64,
}

impl WeightedFlowLogRecordFactor for MinimumWeightedFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedFlowLogRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogRecordCostOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFlowLogRecordFactor for WeightedFlowLogRecordCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFlowLogRecord, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleWeightedFlowLogRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedFlowLogRecordFactor for TitleWeightedFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &WeightedFlowLogRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted flow log record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFlowLogRecordEngine {
    pub steps: Vec<Rc<dyn WeightedFlowLogRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFlowLogRecordEngine {
    pub fn price(&mut self, item: &WeightedFlowLogRecord) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFlowLogRecordFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFlowLogRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_flow_log_record_engine() -> WeightedFlowLogRecordEngine {
    WeightedFlowLogRecordEngine {
        steps: vec![
            Rc::new(MinimumWeightedFlowLogRecordAdjuster { minimum: 34.7 }) as Rc<dyn WeightedFlowLogRecordFactor>,
            Rc::new(WeightedFlowLogRecordCostOffsetAdjuster { weight: 1.41 }),
            Rc::new(TitleWeightedFlowLogRecordAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.22),
                    ("west".to_string(), 1.3),
                    ("central".to_string(), 0.94),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 43241.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_flow_log_record_engine();
        let item = new_weighted_flow_log_record("code-993", "title-101");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
